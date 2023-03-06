use std::fmt::Display;

use games::State;
use more_asserts::debug_assert_ge;

use crate::games;

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
            regret_sum: vec![0.0; actions.len()],
            strategy: vec![0.0; actions.len()],
            strategy_sum: vec![0.0; actions.len()],

            actions,
            info_set,
        }
    }

    pub fn get_actions(&self) -> Vec<S::Action> {
        self.actions.clone()
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

        // How can I prevent cloning the array here?
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

    pub fn add_regret_sum(&mut self, action_index: usize, regret: f64, opponent_prob: f64) {
        self.regret_sum[action_index] += opponent_prob * regret;
    }
}

impl<S> std::cmp::Eq for Node<S> where S: State {}

impl<S> std::cmp::PartialEq for Node<S>
where
    S: State,
{
    fn eq(&self, other: &Self) -> bool {
        self.info_set.eq(&other.info_set)
    }
}

impl<S> std::cmp::PartialOrd for Node<S>
where
    S: State,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.info_set.cmp(&other.info_set))
    }
}

impl<S> std::cmp::Ord for Node<S>
where
    S: State,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.info_set.cmp(&other.info_set)
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
