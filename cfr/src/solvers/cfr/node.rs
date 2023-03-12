use std::fmt::Display;

use games::Game;
use more_asserts::debug_assert_ge;

use crate::games;

pub struct Node<G>
where
    G: Game,
{
    regret_sum: Vec<f64>,
    strategy: Vec<f64>,
    strategy_sum: Vec<f64>,

    actions: Vec<G::Action>,
    info_set: G::InfoSet,
}

impl<G> Node<G>
where
    G: Game,
{
    pub fn new(actions: Vec<G::Action>, info_set: G::InfoSet) -> Self {
        Self {
            regret_sum: vec![0.0; actions.len()],
            strategy: vec![0.0; actions.len()],
            strategy_sum: vec![0.0; actions.len()],

            actions,
            info_set,
        }
    }

    pub fn get_actions(&self) -> &[G::Action] {
        &self.actions
    }

    pub fn regret_matching(&mut self, realization_weight: f64) {
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
    }

    pub fn get_strategy(&self) -> &[f64] {
        &self.strategy
    }

    pub fn to_average_strategy(&self) -> Vec<f64> {
        let normalizing_sum: f64 = self.strategy_sum.iter().sum();
        if normalizing_sum == 0.0 {
            let actions_len = self.strategy.len();
            return vec![1.0 / actions_len as f64; actions_len];
        }
        self.strategy_sum.iter().map(|s| s / normalizing_sum).collect()
    }

    pub fn add_regret_sum(&mut self, action_index: usize, regret: f64, opponent_prob: f64) {
        self.regret_sum[action_index] += opponent_prob * regret;
    }
}

impl<G> std::cmp::Eq for Node<G> where G: Game {}

impl<G> std::cmp::PartialEq for Node<G>
where
    G: Game,
{
    fn eq(&self, other: &Self) -> bool {
        self.info_set.eq(&other.info_set)
    }
}

impl<G> std::cmp::PartialOrd for Node<G>
where
    G: Game,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.info_set.cmp(&other.info_set))
    }
}

impl<G> std::cmp::Ord for Node<G>
where
    G: Game,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.info_set.cmp(&other.info_set)
    }
}

impl<G> Display for Node<G>
where
    G: Game,
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

        /*
        write!(f, " Regret[")?;
        for (i, regret) in self.regret_sum.iter().enumerate() {
            write!(f, "{}: {:.03}, ", self.actions[i], regret)?;
        }
        write!(f, "]")?;
        */

        Ok(())
    }
}
