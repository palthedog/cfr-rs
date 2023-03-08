use crate::{
    eval::Strategy,
    games::{
        PlayerId,
        State,
    },
    solvers::Solver,
};
use clap::Args;
use rand::SeedableRng;
use rand_distr::{
    Distribution,
    WeightedIndex,
};
use wyhash::WyRng;

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use super::node::Node;

#[derive(Args)]
pub struct TrainingArgs {
    #[clap(long, short, value_parser, default_value_t = 42)]
    seed: u64,
}

pub struct Trainer<S>
where
    S: State,
{
    nodes: Rc<RefCell<HashMap<S::InfoSet, Rc<RefCell<Node<S>>>>>>,
    rng: WyRng,
}

impl<S> Trainer<S>
where
    S: State,
{
    pub fn train_one_epoch(&mut self) -> f64 {
        let mut p0_util = 0.0;
        let initial = <S as State>::new_root();
        for traverser in 0..=1 {
            let util = self.sampling(&initial, PlayerId::Player(traverser));
            if traverser == 0 {
                p0_util = util;
            }
        }
        p0_util
    }

    pub fn sampling(&mut self, state: &S, traverser_id: PlayerId) -> f64 {
        if state.is_terminal() {
            return state.get_payouts()[traverser_id.index()];
        }

        let player = state.get_node_player_id();

        if player == PlayerId::Chance {
            // Sample an chance action and traverse its sub-tree.
            let actions = state.list_legal_chance_actions();
            let action = self.sample_action(&actions);
            let next_state = state.with_action(action);
            return self.sampling(&next_state, traverser_id);
        }

        let nodes = Rc::clone(&self.nodes);
        let node = Rc::clone(nodes.borrow_mut().entry(state.to_info_set()).or_insert_with(|| {
            let node = Node::new(state.list_legal_actions());
            Rc::new(RefCell::new(node))
        }));
        let actions;
        let strategy;
        {
            let node_cell = node.borrow();
            actions = node_cell.get_actions().to_vec();
            strategy = node_cell.regret_matching();
        }
        assert_eq!(strategy.len(), actions.len());

        if player == traverser_id {
            let mut act_utils: Vec<f64> = Vec::with_capacity(actions.len());
            let mut util = 0.0;
            // Compute action utils
            for (i, act) in actions.iter().enumerate() {
                let next_state = state.with_action(*act);
                let act_util = self.sampling(&next_state, traverser_id);
                act_utils.push(act_util);
                util += strategy[i] * act_util;
            }

            // Compute sampled counter factual regret values for each action.
            let mut node_mut = node.borrow_mut();
            for (i, act_util) in act_utils.iter().enumerate() {
                let act_regret = act_util - util;
                node_mut.regret_sum[i] += act_regret;
            }
            util
        } else {
            // The current player is not the traverser
            let action_index = self.sample_index(&strategy);
            let action = actions[action_index];
            let next_state = state.with_action(action);
            let util = self.sampling(&next_state, traverser_id);

            // Update strategy sum so that we can calculate average strategy.
            // Note that the average strategy is updated on the opponent’s turns to enforce the
            // unbiasedness of the update to the average strategy.
            // (the reach probability of the current history is biased by the opponent's strategy)
            let mut node_mut = node.borrow_mut();
            for (i, act_prob) in strategy.iter().enumerate() {
                node_mut.strategy_sum[i] += act_prob;
            }

            util
        }
    }

    fn sample_action(&mut self, act_probs: &[(S::Action, f64)]) -> S::Action {
        let dist = WeightedIndex::new(act_probs.iter().map(|p| p.1)).unwrap_or_else(|e| {
            panic!("Invalid weights: e: {} probs: {:?}", e, act_probs);
        });
        let index = dist.sample(&mut self.rng);
        act_probs[index].0
    }

    fn sample_index(&mut self, probs: &[f64]) -> usize {
        let dist = WeightedIndex::new(probs).unwrap_or_else(|e| {
            panic!("Invalid weights: e: {} probs: {:?}", e, probs);
        });
        dist.sample(&mut self.rng)
    }
}

impl<S: State> Strategy<S> for Trainer<S> {
    fn get_strategy(&self, info_set: &<S as State>::InfoSet) -> Vec<f64> {
        self.nodes.borrow().get(info_set).unwrap().borrow().to_average_strategy()
    }
}

impl<G: State> Solver<G> for Trainer<G> {
    type SolverArgs = TrainingArgs;

    fn new(args: Self::SolverArgs) -> Self {
        Trainer {
            rng: WyRng::seed_from_u64(args.seed),
            nodes: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn train_one_epoch(&mut self) -> f64 {
        self.train_one_epoch()
    }

    fn print_strategy(&self) {}
}
