use crate::{
    eval::Strategy,
    games::{Game, PlayerId},
    solvers::Solver,
};
use clap::Args;
use log::info;
use rand::SeedableRng;
use rand_distr::{Distribution, WeightedIndex};
use wyhash::WyRng;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::node::Node;

#[derive(Args)]
pub struct SolverArgs {
    #[clap(long, short, value_parser, default_value_t = 42)]
    seed: u64,
}

impl Default for SolverArgs {
    fn default() -> Self {
        SolverArgs {
            seed: 42,
        }
    }
}

pub struct Trainer<G>
where
    G: Game,
{
    game: G,
    nodes: Rc<RefCell<HashMap<G::InfoSet, Rc<RefCell<Node<G>>>>>>,
    rng: WyRng,

    touched_nodes_count: usize,
}

impl<G> Trainer<G>
where
    G: Game,
{
    pub fn train_one_epoch(&mut self) -> f64 {
        let mut p0_util = 0.0;
        let initial = self.game.new_root();
        for traverser in 0..=1 {
            let util = self.sampling(&initial, PlayerId::Player(traverser));
            if traverser == 0 {
                p0_util = util;
            }
        }
        p0_util
    }

    pub fn sampling(&mut self, state: &G::State, traverser_id: PlayerId) -> f64 {
        self.touched_nodes_count += 1;

        if self.game.is_terminal(state) {
            return self.game.get_payouts(state)[traverser_id.index()];
        }

        let player = self.game.get_node_player_id(state);

        if player == PlayerId::Chance {
            // Sample an chance action and traverse its sub-tree.
            let action = self.game.sample_chance_action(&mut self.rng, state);
            let next_state = self.game.with_action(state, action);
            return self.sampling(&next_state, traverser_id);
        }

        let node = Rc::clone(
            self.nodes.borrow_mut().entry(self.game.to_info_set(state)).or_insert_with(|| {
                let node = Node::new(self.game.list_legal_actions(state));
                Rc::new(RefCell::new(node))
            }),
        );
        let mut node_mut = node.borrow_mut();
        node_mut.regret_matching();
        let strategy = node_mut.get_strategy();
        let actions = node_mut.get_actions();
        debug_assert_eq!(strategy.len(), actions.len());

        if player == traverser_id {
            let mut act_utils: Vec<f64> = Vec::with_capacity(strategy.len());
            let mut util = 0.0;
            // Compute action utils
            for (i, act) in actions.iter().enumerate() {
                let next_state = self.game.with_action(state, *act);
                let act_util = self.sampling(&next_state, traverser_id);
                act_utils.push(act_util);
                util += strategy[i] * act_util;
            }

            // Compute sampled counter factual regret values for each action.
            for (i, act_util) in act_utils.iter().enumerate() {
                let act_regret = act_util - util;
                node_mut.regret_sum[i] += act_regret;
            }
            util
        } else {
            // The current player is not the traverser
            let action_index = self.sample_index(strategy);
            let action = actions[action_index];
            let next_state = self.game.with_action(state, action);
            let util = self.sampling(&next_state, traverser_id);

            // Update strategy sum so that we can calculate average strategy.
            // Note that the average strategy is updated on the opponent’s turns to enforce the
            // unbiasedness of the update to the average strategy.
            // (the reach probability of the current history is biased by the opponent's strategy)
            node_mut.update_strategy_sum();

            util
        }
    }

    fn sample_index(&mut self, probs: &[f64]) -> usize {
        let dist = WeightedIndex::new(probs).unwrap_or_else(|e| {
            panic!("Invalid weights: e: {} probs: {:?}", e, probs);
        });
        dist.sample(&mut self.rng)
    }
}

impl<G: Game> Strategy<G> for Trainer<G> {
    fn get_strategy(&self, info_set: &<G as Game>::InfoSet) -> Option<Vec<f64>> {
        self.nodes.borrow().get(info_set).map(|node| node.borrow().to_average_strategy())
    }
}

impl<G: Game> Solver<G> for Trainer<G> {
    type SolverArgs = SolverArgs;

    fn new(game: G, args: Self::SolverArgs) -> Self {
        Trainer {
            game,
            nodes: Rc::new(RefCell::new(HashMap::new())),
            rng: WyRng::seed_from_u64(args.seed),
            touched_nodes_count: 0,
        }
    }

    fn game_ref(&self) -> &G {
        &self.game
    }

    fn get_touched_nodes_count(&self) -> usize {
        self.touched_nodes_count
    }

    fn train_one_epoch(&mut self) -> f64 {
        self.train_one_epoch()
    }

    fn print_strategy(&self) {
        let nodes = self.nodes.borrow();
        info!("# of nodes: {}", nodes.len());
        /*
        for node in nodes.iter().take(100) {
            info!("  {:?}", node.0);
        }
         */
    }
}
