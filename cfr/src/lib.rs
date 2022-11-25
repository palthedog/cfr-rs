pub mod games;

use std::{
    collections::HashMap,
    fmt::Display,
};

use games::State;
use log::{
    debug,
    info,
};
use more_asserts::{
    assert_gt,
    debug_assert_ge,
};

use crate::games::PlayerId;

#[derive(Clone)]
pub struct Node<S>
where
    S: State,
{
    regret_sum: Vec<f64>,
    strategy: Vec<f64>,
    strategy_sum: Vec<f64>,

    actions: Vec<S::Action>,
    info_set: S::InfoSet,
}

impl<S> Node<S>
where
    S: State,
{
    pub fn new(actions: Vec<S::Action>, info_set: S::InfoSet) -> Self {
        Self {
            regret_sum: vec![],
            strategy: vec![],
            strategy_sum: vec![],

            actions,
            info_set,
        }
    }

    pub fn to_strategy(&mut self, realization_weight: f64) -> Vec<f64> {
        let normalizing_sum: f64 = self.regret_sum.iter().filter(|v| **v >= 0.0).sum();
        let actions_len = self.strategy.len();
        if normalizing_sum == 0.0 {
            self.strategy = vec![1.0 / actions_len as f64; actions_len];
        } else {
            for (i, reg) in self.regret_sum.iter().enumerate() {
                self.strategy[i] = if *reg >= 0.0 {
                    *reg / normalizing_sum
                } else {
                    0.0
                };
            }
        };

        for i in 0..actions_len {
            debug_assert_ge!(self.strategy[i], 0.0);
            self.strategy_sum[i] += realization_weight * self.strategy[i];
        }

        // How can I prevent cloneing the array here?
        self.strategy.clone()
    }

    pub fn to_average_strategy(&self) -> Vec<f64> {
        let normalizing_sum: f64 = self.strategy_sum.iter().sum();
        if normalizing_sum == 0.0 {
            let actions_len = self.strategy.len();
            return vec![1.0 / actions_len as f64; actions_len];
        }
        self.strategy_sum.iter().map(|s| s / normalizing_sum).collect()
    }
}

impl<S> Display for Node<S>
where
    S: State,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Info set
        write!(f, "{}", self.info_set)?;

        // avg Strategy
        let avg_strategy = self.to_average_strategy();

        write!(f, " Avg Strategy[")?;
        for (i, act) in self.actions.iter().enumerate() {
            write!(f, "{}: {:.03}, ", act, avg_strategy[i])?;
        }
        write!(f, "]")?;

        // regrets
        /*
        write!(f, " Regret Sum [")?;
        for i in 0..actions.len() {
            write!(f, "{}: {:.08}, ", actions[i], self.regret_sum[i])?;
        }
        write!(f, "]")?;
        */

        Ok(())
    }
}

pub struct Trainer<S>
where
    S: State,
{
    nodes: HashMap<S::InfoSet, Node<S>>,
}

impl<S> Trainer<S>
where
    S: State,
{
    pub fn new() -> Self {
        Trainer {
            nodes: HashMap::new(),
        }
    }

    pub fn cfr(&mut self, state: &S, actions_prob: [f64; 2]) -> [f64; 2] {
        if state.is_terminal() {
            return state.get_payouts();
        }

        let player = state.get_node_player_id();
        if player == PlayerId::Chance {
            todo!();
        }

        let info_set = state.to_info_set();
        let node = self.nodes.entry(info_set.clone()).or_insert_with(|| {
            let actions = state.list_legal_actions();
            Node::new(actions, info_set.clone())
        });
        let mut node_util = [0.0f64; 2];

        // TODO: avoid cloning actions here.
        let actions = node.actions.clone();
        let actions_len = actions.len();
        assert_gt!(actions_len, 0);
        debug!("CFR state: {:#?}", state);
        debug!("legal actions: {:#?}", node.actions);

        if node.strategy.is_empty() {
            // initialize buffers
            node.strategy.resize(actions_len, 0.0);
            node.strategy_sum.resize(actions_len, 0.0);
            node.regret_sum.resize(actions_len, 0.0);
        }

        let mut action_utils = vec![0.0; actions_len]; // Note: allocating array on the stack is faster.
        let realization_weight = actions_prob[player.index()];
        let strategy = node.to_strategy(realization_weight);
        for (i, act) in actions.iter().enumerate() {
            let action_prob = strategy[i];
            let next_state = state.with_action(*act);
            let mut next_actions_prob = actions_prob;

            next_actions_prob[player.index()] *= action_prob;

            let action_util = self.cfr(&next_state, next_actions_prob);
            action_utils[i] = action_util[player.index()];
            for (player, player_action_util) in action_util.iter().enumerate() {
                node_util[player] += action_prob * player_action_util;
            }
        }

        let opponent = player.opponent();
        let node = self.nodes.get_mut(&info_set).unwrap();
        for i in 0..actions_len {
            let regret: f64 = action_utils[i] - node_util[player.index()];
            let opponent_prob = actions_prob[opponent.index()];
            node.regret_sum[i] += opponent_prob * regret;
        }

        node_util
    }

    pub fn train(&mut self, iterations: u32) {
        let mut rng = rand::thread_rng();
        let mut util = 0.0;
        for i in 0..iterations {
            if i != 0 && i % 10000 == 0 {
                info!("epoch {}: Average game value: {}", i, util / i as f64);
            }
            let initial = <S as State>::new_root(&mut rng);
            util += self.cfr(&initial, [1.0, 1.0])[PlayerId::Player(0).index()];
        }
        info!("Training has finished");

        let nodes: Vec<Node<S>> = self.nodes.values().cloned().collect();
        info!("Nodes [");
        for node in nodes {
            info!("    {}", node);
        }
        info!("]");

        info!("Average game value: {}", util / iterations as f64);
    }
}
