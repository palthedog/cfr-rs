pub mod node;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{eval::Strategy, games::Game};
use clap::Args;
use log::{debug, info};
use more_asserts::assert_gt;
use node::Node;

use crate::games::PlayerId;

use super::Solver;

#[derive(Args)]
pub struct SolverArgs {}

pub struct Trainer<G>
where
    G: Game,
{
    game: G,
    //nodes: HashMap<G::InfoSet, Node<G>>,
    nodes: Rc<RefCell<HashMap<G::InfoSet, Rc<RefCell<Node<G>>>>>>,
    touched_nodes_count: usize,
}

impl<G> Trainer<G>
where
    G: Game,
{
    #[cfg(test)]
    pub fn new_with_nodes(
        game: G,
        _args: SolverArgs,
        nodes: Rc<RefCell<HashMap<G::InfoSet, Rc<RefCell<Node<G>>>>>>,
    ) -> Self {
        Trainer {
            game,
            nodes,
            touched_nodes_count: 0,
        }
    }

    pub fn cfr(&mut self, state: &G::State, actions_prob: [f64; 2]) -> [f64; 2] {
        self.touched_nodes_count += 1;

        if self.game.is_terminal(state) {
            return self.game.get_payouts(state);
        }

        let player = self.game.get_node_player_id(state);
        if player == PlayerId::Chance {
            let actions = self.game.list_legal_chance_actions(state);
            let mut node_util = [0.0, 0.0];
            for (act, prob) in actions {
                let next_state = self.game.with_action(state, act);
                let mut next_actions_prob = actions_prob;
                for action_prob in &mut next_actions_prob {
                    *action_prob *= prob;
                }
                let action_util = self.cfr(&next_state, next_actions_prob);
                for (player, player_action_util) in action_util.iter().enumerate() {
                    node_util[player] += prob * player_action_util;
                }
            }
            return node_util;
        }

        let info_set = self.game.to_info_set(state);
        let node = Rc::clone(
            self.nodes.borrow_mut().entry(self.game.to_info_set(state)).or_insert_with(|| {
                let actions = self.game.list_legal_actions(state);
                let node = Node::new(actions, info_set.clone());
                Rc::new(RefCell::new(node))
            }),
        );
        let mut node_mut = node.borrow_mut();
        let mut node_util = [0.0f64; 2];

        let actions_len = node_mut.get_actions().len();
        assert_gt!(actions_len, 0);
        debug!("CFR state: {:#?}", state);
        debug!("legal actions: {:#?}", node_mut.get_actions());

        let mut player_action_utils = vec![0.0; actions_len]; // Note: allocating array on the stack is faster.
        let realization_weight = actions_prob[player.index()];
        node_mut.regret_matching(realization_weight);
        let strategy = node_mut.get_strategy();
        for (i, act) in node_mut.get_actions().iter().enumerate() {
            let action_prob = strategy[i];
            let next_state = self.game.with_action(state, *act);
            let mut next_actions_prob = actions_prob;

            next_actions_prob[player.index()] *= action_prob;

            let action_util = self.cfr(&next_state, next_actions_prob);
            player_action_utils[i] = action_util[player.index()];
            for (player, action_util) in action_util.iter().enumerate() {
                node_util[player] += action_prob * action_util;
            }
        }

        let opponent = player.opponent();
        for (i, action_util) in player_action_utils.iter().enumerate() {
            let regret: f64 = action_util - node_util[player.index()];
            let opponent_prob = actions_prob[opponent.index()];
            node_mut.add_regret_sum(i, regret, opponent_prob);
        }

        node_util
    }

    fn train_one_epoch(&mut self) -> f64 {
        let initial = self.game.new_root();
        self.cfr(&initial, [1.0, 1.0])[PlayerId::Player(0).index()]
    }

    fn print_nodes(&self) {
        let nodes = self.nodes.borrow();
        let mut nodes: Vec<_> = nodes.values().collect();
        nodes.sort();
        info!("Nodes [");
        for node in nodes {
            info!("    {}", node.borrow());
        }
        info!("]");
    }
}

impl<G: Game> Strategy<G> for Trainer<G> {
    fn get_strategy(&self, info_set: &<G as Game>::InfoSet) -> Option<Vec<f64>> {
        Some(self.nodes.borrow().get(info_set).unwrap().borrow().to_average_strategy())
    }
}

impl<G: Game> Solver<G> for Trainer<G> {
    type SolverArgs = SolverArgs;

    fn new(game: G, _args: Self::SolverArgs) -> Self {
        Trainer {
            game,
            nodes: Rc::new(RefCell::new(HashMap::new())),
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
        self.print_nodes();
    }
}
