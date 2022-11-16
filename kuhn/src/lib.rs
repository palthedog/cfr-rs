use std::{
    collections::HashMap,
    fmt::Display,
};

use log::info;
use more_asserts::assert_ge;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Card {
    Jack = 0,
    Queen = 1,
    King = 2,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Pass,
    Bet,
}

impl Action {
    const VALUES: [Action; 2] = [Action::Pass, Action::Bet];
    const COUNT: usize = Action::VALUES.len();
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InfoSet {
    pub player_id: usize,
    pub actions: [Option<Action>; 2],
    pub card: Card,
}

impl From<&State> for InfoSet {
    fn from(state: &State) -> Self {
        InfoSet {
            player_id: state.next_player_id,
            actions: state.actions.clone(),
            card: state.cards[state.next_player_id],
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub next_player_id: usize,
    pub actions: [Option<Action>; 2],
    pub cards: [Card; 2],
    pub pot: i32,
}

impl State {
    pub fn new(cards: [Card; 2]) -> Self {
        Self {
            next_player_id: 0,
            actions: [None, None],
            cards,
            pot: 2, // ante
        }
    }

    pub fn with_action(&self, action: Action) -> Self {
        let mut next = self.clone();
        next.next_player_id = self.next_player_id ^ 1;
        next.actions[self.next_player_id] = Some(action);
        if action == Action::Bet {
            next.pot += 1;
        }
        next
    }

    pub fn get_payout_for_next_player(&self) -> i32 {
        let p = self.next_player_id;
        let o = self.next_player_id ^ 1;
        if self.actions[p] == Some(Action::Bet) && self.actions[o] == Some(Action::Pass) {
            // opponent folded
            return 1;
        }

        let win = self.cards[p] > self.cards[o];
        match (self.actions[p], self.actions[o]) {
            (Some(Action::Pass), Some(Action::Bet)) => -1, // ante
            (Some(Action::Bet), Some(Action::Pass)) => 1,
            (Some(Action::Pass), Some(Action::Pass)) => {
                if win {
                    1
                } else {
                    -1
                }
            }
            (Some(Action::Bet), Some(Action::Bet)) => {
                if win {
                    2
                } else {
                    -2
                }
            }
            _ => panic!(),
        }
    }

    pub fn is_terminal(&self) -> bool {
        if self.actions[self.next_player_id] == Some(Action::Bet)
            && self.actions[self.next_player_id ^ 1] == Some(Action::Pass)
        {
            // opponent folded
            return true;
        }
        self.actions.iter().all(|a| *a == Some(Action::Pass))
            || self.actions.iter().all(|a| *a == Some(Action::Bet))
    }
}

#[derive(Clone)]
pub struct Node {
    regret_sum: Vec<f64>,
    strategy: Vec<f64>,
    strategy_sum: Vec<f64>,

    info_set: InfoSet,
}

impl Node {
    pub fn new(info_set: InfoSet) -> Self {
        Self {
            regret_sum: vec![0.0; Action::COUNT],
            strategy: vec![0.0; Action::COUNT],
            strategy_sum: vec![0.0; Action::COUNT],
            info_set,
        }
    }

    pub fn to_strategy(&mut self, realization_weight: f64) -> Vec<f64> {
        let positive_regret_sum: Vec<f64> = self.regret_sum.iter().map(|v| v.max(0.0)).collect();
        let normalizing_sum: f64 = positive_regret_sum.iter().sum();
        self.strategy = if normalizing_sum == 0.0 {
            vec![1.0 / Action::COUNT as f64; Action::COUNT]
        } else {
            positive_regret_sum
                .iter()
                .map(|reg| *reg / normalizing_sum)
                .collect()
        };

        for i in 0..Action::COUNT {
            assert_ge!(self.strategy[i], 0.0);
            self.strategy_sum[i] += realization_weight * self.strategy[i];
        }

        // How can I prevent cloneing the array here?
        self.strategy.clone()
    }

    pub fn to_average_strategy(&self) -> Vec<f64> {
        let normalizing_sum: f64 = self.strategy_sum.iter().sum();
        if normalizing_sum == 0.0 {
            return vec![1.0 / Action::COUNT as f64; Action::COUNT];
        }
        self.strategy_sum
            .iter()
            .map(|s| s / normalizing_sum)
            .collect()
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node({} {:5}, [{:11},{:11}]): ",
            self.info_set.player_id,
            format!("{:?}", self.info_set.card),
            format!("{:?}", self.info_set.actions[0]),
            format!("{:?}", self.info_set.actions[1]),
        )?;

        let avg_s = self.to_average_strategy();
        write!(
            f,
            "{:5?}: {:.4}, {:5?}: {:.4}",
            Action::VALUES[0],
            avg_s[0],
            Action::VALUES[1],
            avg_s[1]
        )?;

        Ok(())
    }
}

pub struct Trainer {
    nodes: HashMap<InfoSet, Node>,
}

impl Trainer {
    pub fn new() -> Self {
        Trainer {
            nodes: HashMap::new(),
        }
    }

    pub fn cfr(&mut self, state: &State, actions_prob: [f64; 2]) -> f64 {
        if state.is_terminal() {
            return state.get_payout_for_next_player() as f64;
        }

        let player = state.next_player_id;
        let opponent = state.next_player_id ^ 1;

        let info_set = InfoSet::from(state);
        let node = self
            .nodes
            .entry(info_set.clone())
            .or_insert_with(|| Node::new(info_set.clone()));

        let mut node_util: f64 = 0.0;
        let mut action_utils = [0.0; Action::COUNT];
        let realization_weight = actions_prob[player];
        let strategy = node.to_strategy(realization_weight);
        for i in 0..Action::COUNT {
            let act = Action::VALUES[i];
            let action_prob = strategy[i];

            let next_state = state.with_action(act);
            let mut next_actions_prob = actions_prob;
            next_actions_prob[player] *= action_prob;

            action_utils[i] = -self.cfr(&next_state, next_actions_prob);
            node_util += action_prob * action_utils[i];
        }

        let node = self.nodes.get_mut(&info_set).unwrap();
        for i in 0..Action::COUNT {
            let regret: f64 = action_utils[i] - node_util;
            let opponent_prob = actions_prob[opponent];
            node.regret_sum[i] += opponent_prob * regret;
        }

        node_util
    }

    pub fn train(&mut self, iterations: u32) {
        let mut rng = rand::thread_rng();
        let mut util = 0.0;
        for _i in 0..iterations {
            let mut cards = [Card::Jack, Card::Queen, Card::King];
            cards.shuffle(&mut rng);
            let initial = State::new([cards[0], cards[1]]);
            util += self.cfr(&initial, [1.0, 1.0]);
        }
        info!("Training has finished");
        info!("Average game value: {}", util / iterations as f64);

        let mut nodes: Vec<Node> = self.nodes.values().cloned().collect();
        nodes.sort_by_key(|n| (n.info_set.actions, n.info_set.card));
        info!("Nodes [");
        for node in nodes {
            info!("    {}", node);
        }
        info!("]");
    }
}
