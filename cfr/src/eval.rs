use std::collections::HashMap;

use itertools::Itertools;
use log::debug;
use more_asserts::assert_ge;

use crate::games::{
    GameState,
    PlayerId,
};

pub trait Strategy<S: GameState> {
    // TODO: Make it alloc-free.
    // e.g.
    //  - sample(rng, actions: &[Action]) -> Action
    //  - get_strategy(buf: &mut &[f64])
    fn get_strategy(&self, info_set: &S::InfoSet) -> Option<Vec<f64>>;

    fn safe_get_strategy(&self, actions_len: usize, info_set: &S::InfoSet) -> Vec<f64> {
        match self.get_strategy(&info_set) {
            Some(s) => s,
            None => vec![1.0 / actions_len as f64; actions_len],
        }
    }
}

fn max_index(values: &[f64]) -> usize {
    values.iter().enumerate().max_by(|(_i, a), (_j, b)| a.total_cmp(b)).map(|(i, _)| i).unwrap()
}

fn best_response_utils_to_pure_strategy<G: GameState>(
    values: &HashMap<G::InfoSet, Vec<f64>>,
) -> HashMap<G::InfoSet, Vec<f64>> {
    values
        .iter()
        .map(|e| {
            let utils = e.1;
            let index = max_index(utils);
            let mut pure_strategy = vec![0.0; utils.len()];
            pure_strategy[index] = 1.0;
            (e.0.clone(), pure_strategy)
        })
        .collect()
}

impl<S: GameState> Strategy<S> for HashMap<S::InfoSet, Vec<f64>> {
    fn get_strategy(&self, info_set: &<S as GameState>::InfoSet) -> Option<Vec<f64>> {
        Some(self.get(info_set).unwrap().clone())
    }
}

pub struct ReachProbabilities<S: GameState> {
    reach_probabilities: HashMap<S, f64>,
}

impl<S: GameState> ReachProbabilities<S> {
    fn insert(&mut self, state: S, reach_probability: f64) {
        let prob = self.reach_probabilities.entry(state).or_insert(0.0);
        *prob += reach_probability;
    }
}

impl<S: GameState> Default for ReachProbabilities<S> {
    fn default() -> Self {
        Self {
            reach_probabilities: HashMap::new(),
        }
    }
}

pub fn calc_reach_probabilities<S: GameState, St: Strategy<S>>(
    br_player_id: PlayerId,
    strategy: &St,
    state: &S,
    reach_probability: f64,
    reach_probabilities: &mut HashMap<S::InfoSet, ReachProbabilities<S>>,
) {
    if state.is_terminal() {
        return;
    }

    let node_player_id = state.get_node_player_id();
    if node_player_id == PlayerId::Chance {
        let actions = state.list_legal_chance_actions();
        for (act, prob) in actions {
            let next_state = state.with_action(act);
            calc_reach_probabilities(
                br_player_id,
                strategy,
                &next_state,
                reach_probability * prob,
                reach_probabilities,
            );
        }
        return;
    }

    let info_set = state.to_info_set();
    let actions = state.list_legal_actions();
    if node_player_id == br_player_id {
        // the player plays the best response.
        let stateful_info_set = reach_probabilities.entry(info_set).or_default();
        stateful_info_set.insert(state.clone(), reach_probability);

        for act in actions {
            let next_state = state.with_action(act);
            calc_reach_probabilities(
                br_player_id,
                strategy,
                &next_state,
                reach_probability * 1.0, // br_player always choose the best action.
                reach_probabilities,
            );
        }
    } else {
        // the opponent player plays as the trained strategy.
        let strategy_ary = strategy.safe_get_strategy(actions.len(), &info_set);
        for (i, act) in actions.iter().enumerate() {
            let prob = strategy_ary[i];
            let next_state = state.with_action(*act);
            calc_reach_probabilities(
                br_player_id,
                strategy,
                &next_state,
                reach_probability * prob,
                reach_probabilities,
            );
        }
    }
}

/// Calculate an expected utility value at the given `state` if:
/// - the `br_player` plays the best hand (the player knows opponent's strategy)
/// - other players play the trained strategies by `trainer`
pub fn calc_best_response_value<S: GameState, St: Strategy<S>>(
    action_utilities: &mut HashMap<S::InfoSet, Vec<f64>>,
    reach_probabilities: &HashMap<S::InfoSet, ReachProbabilities<S>>,
    br_player: PlayerId,
    strategy: &St,
    state: &S,
) -> f64 {
    if state.is_terminal() {
        return state.get_payouts()[br_player.index()];
    }

    if state.get_node_player_id() == PlayerId::Chance {
        let actions = state.list_legal_chance_actions();
        let mut node_util = 0.0;
        for (act, prob) in actions {
            let next_state = state.with_action(act);
            let act_util = calc_best_response_value(
                action_utilities,
                reach_probabilities,
                br_player,
                strategy,
                &next_state,
            );
            node_util += prob * act_util;
        }
        return node_util;
    }

    let actions = state.list_legal_actions();
    if state.get_node_player_id() == br_player {
        // Node for the Best Response player.
        // The player always plays the best response for the current info set.

        let info_set = state.to_info_set();

        // Check the best action for the current INFO SET (not `state`)
        if !action_utilities.contains_key(&info_set) {
            let mut act_utils = vec![0.0; actions.len()];
            for (act_i, act) in actions.iter().enumerate() {
                let rp = reach_probabilities.get(&info_set).unwrap();
                let mut act_util = 0.0;
                for (sib_state, state_reach_prob) in rp.reach_probabilities.iter() {
                    let next_state = sib_state.with_action(*act);
                    let util = calc_best_response_value(
                        action_utilities,
                        reach_probabilities,
                        br_player,
                        strategy,
                        &next_state,
                    );
                    act_util += state_reach_prob * util;
                }
                act_utils[act_i] = act_util;
            }
            action_utilities.insert(info_set.clone(), act_utils);
        }

        // Play the best response for the current STATE.
        let best_action_index = max_index(action_utilities.get(&info_set).unwrap());
        let best_action = actions[best_action_index];
        let next_state = state.with_action(best_action);
        return calc_best_response_value(
            action_utilities,
            reach_probabilities,
            br_player,
            strategy,
            &next_state,
        );
    }

    // the opponent player plays as the trained strategy.
    let info_set = state.to_info_set();
    let strategy_ary = strategy.safe_get_strategy(actions.len(), &info_set);
    let mut node_util = 0.0;
    for (i, act) in actions.iter().enumerate() {
        let act_prob = strategy_ary[i];

        let next_state = state.with_action(*act);
        let util = calc_best_response_value(
            action_utilities,
            reach_probabilities,
            br_player,
            strategy,
            &next_state,
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
    S: GameState,
    S0: Strategy<S>,
    S1: Strategy<S>,
{
    if state.is_terminal() {
        return state.get_payouts()[player_id.index()];
    }
    if state.get_node_player_id() == PlayerId::Chance {
        let actions = state.list_legal_chance_actions();
        let mut node_util = 0.0;
        for (act, prob) in actions {
            let next_state = state.with_action(act);
            let action_util = calc_expected_value(player_id, strategy0, strategy1, &next_state);
            node_util += prob * action_util;
        }
        return node_util;
    }
    let actions = state.list_legal_actions();
    let info_set = state.to_info_set();
    let strategy = match state.get_node_player_id() {
        PlayerId::Player(0) => strategy0.safe_get_strategy(actions.len(), &info_set),
        PlayerId::Player(1) => strategy1.safe_get_strategy(actions.len(), &info_set),
        PlayerId::Player(_) => panic!(),
        PlayerId::Chance => todo!(),
    };
    debug!("p: {:?}, infoset: {}, strategy: {:?}", state.get_node_player_id(), info_set, strategy);
    let mut ev = 0.0;
    for (i, act) in actions.iter().enumerate() {
        let act_value =
            calc_expected_value(player_id, strategy0, strategy1, &state.with_action(*act));
        let prob = strategy[i];
        ev += prob * act_value;
    }
    ev
}

pub fn compute_exploitability<G: GameState, St: Strategy<G>>(strategy: &St) -> f64 {
    let root_state = G::new_root();
    let mut rp0: HashMap<G::InfoSet, ReachProbabilities<G>> = HashMap::new();
    let mut rp1: HashMap<G::InfoSet, ReachProbabilities<G>> = HashMap::new();
    calc_reach_probabilities(PlayerId::Player(0), strategy, &root_state, 1.0, &mut rp0);
    calc_reach_probabilities(PlayerId::Player(1), strategy, &root_state, 1.0, &mut rp1);
    let mut brmap0: HashMap<G::InfoSet, Vec<f64>> = HashMap::new();
    let mut brmap1: HashMap<G::InfoSet, Vec<f64>> = HashMap::new();
    debug!("Calculating best response for player 0");
    let br0 =
        calc_best_response_value(&mut brmap0, &rp0, PlayerId::Player(0), strategy, &root_state);
    debug!("Calculating best response for player 1");
    let br1 =
        calc_best_response_value(&mut brmap1, &rp1, PlayerId::Player(1), strategy, &root_state);
    debug!("util_0(br0): {}, util_1(br1): {}", br0, br1);

    let br_pure_strategies0 = best_response_utils_to_pure_strategy::<G>(&brmap0);
    let br_pure_strategies1 = best_response_utils_to_pure_strategy::<G>(&brmap1);

    if log::log_enabled!(log::Level::Debug) {
        debug!("Best responses for Player0");
        for info_set in brmap0.keys().sorted() {
            let br = brmap0.get(info_set).unwrap();
            debug!("{}: {:?}", info_set, br);
            let rp = rp0.get(info_set).unwrap();
            for (s, prob) in rp.reach_probabilities.iter().sorted_by_key(|(k, _v)| *k) {
                debug!("    {:?}: {}", s, prob);
            }
        }
        debug!("Best responses for Player1");
        for info_set in brmap1.keys().sorted() {
            let br = brmap1.get(info_set).unwrap();
            debug!("{}: {:?}", info_set, br);
            let rp = rp1.get(info_set).unwrap();
            for (s, prob) in rp.reach_probabilities.iter().sorted_by_key(|(k, _v)| *k) {
                debug!("    {:?}: {}", s, prob);
            }
        }
    }
    let root_state = G::new_root();
    let ev_0 =
        calc_expected_value(PlayerId::Player(1), strategy, &br_pure_strategies1, &root_state);
    let ev_1 =
        calc_expected_value(PlayerId::Player(0), &br_pure_strategies0, strategy, &root_state);

    debug!("util_1(s0, s_br1): {} util_0(s_br0, s1): {}", ev_0, ev_1);
    let exploitability = (ev_0 + ev_1) / 2.0;
    assert_ge!(exploitability, 0.0, "Exploitability must be positive value.");
    exploitability
}
