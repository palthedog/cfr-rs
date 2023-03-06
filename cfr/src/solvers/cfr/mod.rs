pub mod node;

use std::{
    collections::HashMap,
    fs::File,
    io::{
        BufWriter,
        Write,
    },
    time::{
        Duration,
        Instant,
    },
};

use std::path::PathBuf;

use clap::{
    Args,
    ValueHint,
};

use crate::{
    eval::Strategy,
    games::State,
};
use log::{
    debug,
    info,
};
use more_asserts::assert_gt;
use node::Node;

use crate::{
    eval::compute_exploitability,
    games::PlayerId,
};

#[derive(Args)]
pub struct TrainingArgs {
    #[clap(long, short, value_parser, default_value_t = 1000)]
    iterations: usize,

    #[clap(long, short, value_parser, value_hint(ValueHint::FilePath))]
    log_path: Option<PathBuf>,
}

impl TrainingArgs {
    pub fn new(iterations: usize) -> Self {
        TrainingArgs {
            iterations,
            log_path: None,
        }
    }
}

pub struct Trainer<S>
where
    S: State,
{
    nodes: HashMap<S::InfoSet, Node<S>>,
}

impl<S> Default for Trainer<S>
where
    S: State,
{
    fn default() -> Self {
        Self::new()
    }
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

    #[cfg(test)]
    pub fn new_with_nodes(nodes: HashMap<S::InfoSet, Node<S>>) -> Self {
        Trainer {
            nodes,
        }
    }

    pub fn cfr(&mut self, state: &S, actions_prob: [f64; 2]) -> [f64; 2] {
        if state.is_terminal() {
            return state.get_payouts();
        }

        let player = state.get_node_player_id();
        if player == PlayerId::Chance {
            let actions = state.list_legal_chance_actions();
            let mut node_util = [0.0, 0.0];
            for (act, prob) in actions {
                let next_state = state.with_action(act);
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

        let info_set = state.to_info_set();
        let node = self.nodes.entry(info_set.clone()).or_insert_with(|| {
            let actions = state.list_legal_actions();
            Node::new(actions, info_set.clone())
        });
        let mut node_util = [0.0f64; 2];

        let actions = node.get_actions();
        let actions_len = actions.len();
        assert_gt!(actions_len, 0);
        debug!("CFR state: {:#?}", state);
        debug!("legal actions: {:#?}", node.get_actions());

        let mut player_action_utils = vec![0.0; actions_len]; // Note: allocating array on the stack is faster.
        let realization_weight = actions_prob[player.index()];
        let strategy = node.to_strategy(realization_weight);
        for (i, act) in actions.iter().enumerate() {
            let action_prob = strategy[i];
            let next_state = state.with_action(*act);
            let mut next_actions_prob = actions_prob;

            next_actions_prob[player.index()] *= action_prob;

            let action_util = self.cfr(&next_state, next_actions_prob);
            player_action_utils[i] = action_util[player.index()];
            for (player, action_util) in action_util.iter().enumerate() {
                node_util[player] += action_prob * action_util;
            }
        }

        let opponent = player.opponent();
        let node = self.nodes.get_mut(&info_set).unwrap();
        for (i, action_util) in player_action_utils.iter().enumerate() {
            let regret: f64 = action_util - node_util[player.index()];
            let opponent_prob = actions_prob[opponent.index()];
            node.add_regret_sum(i, regret, opponent_prob);
        }

        node_util
    }

    pub fn train(&mut self, args: &TrainingArgs) {
        let mut log_writer = if let Some(path) = &args.log_path {
            let f = File::create(path).unwrap_or_else(|err| {
                panic!("Failed to create a file: {:?}, {}", path, err);
            });
            let mut w = BufWriter::new(f);
            writeln!(w, "epoch,elapsed_seconds,exploitability").expect("Failed to write");
            Some(w)
        } else {
            None
        };

        let mut util = 0.0;
        let start_t = Instant::now();
        let mut timer = Instant::now();
        for i in 0..args.iterations {
            let initial = <S as State>::new_root();
            util += self.cfr(&initial, [1.0, 1.0])[PlayerId::Player(0).index()];
            if timer.elapsed() > Duration::from_secs(5) {
                let exploitability = compute_exploitability(self);
                info!("epoch {:10}: exploitability: {}", i, compute_exploitability(self));
                info!("Average game value: {}", util / i as f64);

                if let Some(w) = &mut log_writer {
                    writeln!(w, "{},{},{:.12}", i, start_t.elapsed().as_secs(), exploitability)
                        .expect("Failed to write");
                    w.flush().expect("Failed to flush");
                }

                timer = Instant::now();
            }
        }
        info!("Training has finished");

        let mut nodes: Vec<&Node<S>> = self.nodes.values().collect();
        nodes.sort();
        info!("Nodes [");
        for node in nodes {
            info!("    {}", node);
        }
        info!("]");

        info!("# of infoset: {}", self.nodes.len());
        info!("Average game value: {}", util / args.iterations as f64);
        info!("exploitability: {}", compute_exploitability(self));
    }
}

impl<S: State> Strategy<S> for Trainer<S> {
    fn get_strategy(&self, info_set: &<S as State>::InfoSet) -> Vec<f64> {
        self.nodes.get(info_set).unwrap().to_average_strategy()
    }
}
