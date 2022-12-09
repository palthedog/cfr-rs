use std::collections::HashMap;

use log::debug;

use crate::{
    games::{
        PlayerId,
        State,
    },
    Trainer,
};

pub trait Strategy<S: State> {
    fn get_strategy(&self, info_set: &S::InfoSet) -> Vec<f64>;
}

impl<S: State> Strategy<S> for HashMap<S::InfoSet, Vec<f64>> {
    fn get_strategy(&self, info_set: &<S as State>::InfoSet) -> Vec<f64> {
        // pure strategy
        let utils = self.get(info_set).unwrap();
        let max_index: usize = utils
            .iter()
            .enumerate()
            .max_by(|(_i, a), (_j, b)| a.total_cmp(b))
            .map(|(i, _)| i)
            .unwrap();
        let mut s = vec![0.0; utils.len()];
        s[max_index] = 1.0;
        s
    }
}

impl<S: State> Strategy<S> for Trainer<S> {
    fn get_strategy(&self, info_set: &<S as State>::InfoSet) -> Vec<f64> {
        self.nodes.get(info_set).unwrap().to_average_strategy()
    }
}

/// Calculate the best response for a player `br_player` and return the utilities for the `br_player` at the `state`.
///
/// A player `br_player` plays best response and other players play the strategy computed/stored in `trainer`.
/// * `br_player` - a PlayerId for the player who plays best response
/// * `trainer` - pre-trained game tree
/// * `state` - the current state for where the function calculate the best response
/// * `opponent_probability` - the counterfactual reach probability of the current state.
pub fn calc_best_response_value<S: State>(
    action_utilities: &mut HashMap<S::InfoSet, Vec<f64>>,
    br_player: PlayerId,
    trainer: &Trainer<S>,
    state: &S,
    opponent_probability: f64,
) -> f64 {
    if state.is_terminal() {
        return state.get_payouts()[br_player.index()];
    }

    let info_set = state.to_info_set();
    let node = trainer.nodes.get(&info_set).unwrap();
    let actions = node.get_actions();
    if state.get_node_player_id() == br_player {
        // the player plays the best response.
        let mut act_utils = vec![0.0; actions.len()];
        if !action_utilities.contains_key(&info_set) {
            action_utilities.insert(info_set.clone(), vec![0.0; actions.len()]);
        }

        for (i, act) in actions.iter().enumerate() {
            let next_state = state.with_action(*act);
            let util = calc_best_response_value(
                action_utilities,
                br_player,
                trainer,
                &next_state,
                opponent_probability,
            );
            act_utils[i] = util;

            let mut_act_utils = action_utilities.get_mut(&info_set).unwrap();
            mut_act_utils[i] += opponent_probability * util;
        }
        return *act_utils.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
    }

    // the opponent player plays as the trained strategy.
    // the player plays the best response.
    let strategy = node.to_average_strategy();
    let mut node_util = 0.0;
    for (i, act) in actions.iter().enumerate() {
        let act_prob = strategy[i];
        let next_state = state.with_action(*act);
        let util = calc_best_response_value(
            action_utilities,
            br_player,
            trainer,
            &next_state,
            opponent_probability * act_prob,
        );
        node_util += act_prob * util;
    }
    node_util
}

pub fn calc_expected_value<S, S0, S1>(
    player_id: PlayerId,
    strategy0: &S0,
    strategy1: &S1,
    state: &S,
) -> f64
where
    S: State,
    S0: Strategy<S>,
    S1: Strategy<S>,
{
    if state.is_terminal() {
        return state.get_payouts()[player_id.index()];
    }
    let info_set = state.to_info_set();
    let strategy = match state.get_node_player_id() {
        PlayerId::Player(0) => strategy0.get_strategy(&info_set),
        PlayerId::Player(1) => strategy1.get_strategy(&info_set),
        PlayerId::Player(_) => panic!(),
        PlayerId::Chance => todo!(),
    };
    debug!("p: {:?}, infoset: {}, strategy: {:?}", state.get_node_player_id(), info_set, strategy);
    let mut ev = 0.0;
    for (i, act) in state.list_legal_actions().iter().enumerate() {
        let act_value =
            calc_expected_value(player_id, strategy0, strategy1, &state.with_action(*act));
        let prob = strategy[i];
        ev += prob * act_value;
    }
    ev
}

pub fn compute_exploitability<S: State>(trainer: &Trainer<S>) -> f64 {
    // TODO: do this by chance node...
    let all_root_states = S::list_possible_root_states();
    let mut br0: HashMap<S::InfoSet, Vec<f64>> = HashMap::new();
    let mut br1: HashMap<S::InfoSet, Vec<f64>> = HashMap::new();
    for s in all_root_states.iter() {
        calc_best_response_value(&mut br0, PlayerId::Player(0), trainer, &s, 1.0);
        calc_best_response_value(&mut br1, PlayerId::Player(1), trainer, &s, 1.0);
    }

    let mut exploitability = 0.0;
    for s in all_root_states.iter() {
        let ev_0 = calc_expected_value(PlayerId::Player(0), trainer, &br1, s);
        let ev_1 = calc_expected_value(PlayerId::Player(1), &br0, trainer, s);
        debug!("{:?}: ev0: {} ev1:{}", s, ev_0, ev_1);
        exploitability += (ev_0 + ev_1) / all_root_states.len() as f64;
    }
    exploitability
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::collections::HashMap;

    use crate::{
        games::{
            kuhn::{
                self,
                Card,
                KuhnAction,
            },
            PlayerId,
            State,
        },
        Node,
        Trainer,
    };

    fn new_test_trainer<S: State>(state: &S, node: Node<S>) -> Trainer<S> {
        let mut hashmap = HashMap::new();
        hashmap.insert(state.to_info_set(), node);
        Trainer::new_with_nodes(hashmap)
    }

    #[test]
    fn test_calc_best_response_value_leaf_win() {
        let state = kuhn::KuhnState {
            next_player_id: PlayerId::Player(1),
            actions: [Some(KuhnAction::Bet), None],
            cards: [Card::Queen, Card::King],
            pot: 3,
        };
        assert_eq!(vec![KuhnAction::Pass, KuhnAction::Bet], state.list_legal_actions());
        let node = Node::<kuhn::KuhnState> {
            regret_sum: vec![1.0, 0.0],
            strategy: vec![1.0, 0.0],
            strategy_sum: vec![1.0, 0.0],
            actions: state.list_legal_actions(),
            info_set: state.to_info_set(),
        };
        let trainer = new_test_trainer(&state, node);

        let mut best_responses = HashMap::new();
        assert_eq!(
            2.0,
            calc_best_response_value(
                &mut best_responses,
                PlayerId::Player(1),
                &trainer,
                &state,
                1.0,
            )
        );
    }

    #[test]
    fn test_calc_best_response_value_leaf_lose() {
        let state = kuhn::KuhnState {
            next_player_id: PlayerId::Player(1),
            actions: [Some(KuhnAction::Bet), None],
            cards: [Card::Queen, Card::Jack],
            pot: 3,
        };
        assert_eq!(vec![KuhnAction::Pass, KuhnAction::Bet], state.list_legal_actions());
        let node = Node::<kuhn::KuhnState> {
            regret_sum: vec![1.0, 0.0],
            strategy: vec![1.0, 0.0],
            strategy_sum: vec![1.0, 0.0],
            actions: state.list_legal_actions(),
            info_set: state.to_info_set(),
        };
        let trainer = new_test_trainer(&state, node);

        let mut best_responses = HashMap::new();
        assert_eq!(
            -1.0,
            calc_best_response_value(
                &mut best_responses,
                PlayerId::Player(1),
                &trainer,
                &state,
                1.0,
            )
        );
    }
}
