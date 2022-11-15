use std::{
    cmp::Ordering,
    fmt::Display,
};

use log::info;
use rand::{
    distributions::WeightedIndex,
    prelude::Distribution,
    Rng,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub index: usize,
    pub assignments: Vec<u32>,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for a in &self.assignments {
            write!(f, "{},", a)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

pub struct Trainer {
    valid_actions: Vec<Action>,
    player_regret: Regret,
    opponent_regret: Regret,
}

impl Trainer {
    pub fn new(soldiers_count: u32, battlefields_count: u32) -> Self {
        let valid_actions = Self::list_valid_actions(soldiers_count, battlefields_count);
        let player_regret = Regret::new(valid_actions.len());
        let opponent_regret = Regret::new(valid_actions.len());
        Self {
            valid_actions,
            player_regret,
            opponent_regret,
        }
    }

    pub fn train(&mut self, iterations: u32) {
        let mut rng = rand::thread_rng();
        for i in 0..iterations {
            // Get random action according to mixed-strategy distribution
            let player_strategy = self.player_regret.to_strategy();
            let opponent_strategy = self.opponent_regret.to_strategy();

            self.player_regret.update_strategy_sum(&player_strategy);
            self.opponent_regret.update_strategy_sum(&opponent_strategy);

            let player_action =
                self.player_regret
                    .get_action(&mut rng, &player_strategy, &self.valid_actions);
            let opponent_action =
                self.opponent_regret
                    .get_action(&mut rng, &opponent_strategy, &self.valid_actions);

            self.player_regret
                .update_regret(&player_action, &opponent_action, &self.valid_actions);
            self.opponent_regret.update_regret(
                &opponent_action,
                &player_action,
                &self.valid_actions,
            );

            if i % 1000 == 0 {
                self.print_avg_strategy();
            }
        }
        self.print_avg_strategy();
    }

    pub fn print_avg_strategy(&self) {
        let avg_strategy = self.player_regret.to_average_strategy();
        let mut s = "Avg-Strategy [\n".to_string();
        s += &format!("  {:8}    {}\n", "Strategy", "Probability");
        for (i, act) in self.valid_actions.iter().enumerate() {
            s += &format!("  {:8}    {:.05}\n", act, avg_strategy[i]);
        }
        s += "]";
        info!("{}", s);
    }

    fn list_valid_actions(soldiers_count: u32, battlefields_count: u32) -> Vec<Action> {
        Self::list_valid_actions_impl(soldiers_count, battlefields_count)
            .into_iter()
            .enumerate()
            .map(|(i, v)| Action {
                index: i,
                assignments: v,
            })
            .collect()
    }

    fn list_valid_actions_impl(soldiers_count: u32, battlefields_count: u32) -> Vec<Vec<u32>> {
        if battlefields_count == 1 {
            return vec![vec![soldiers_count]];
        }
        let mut ret = vec![];
        for s in 0..soldiers_count + 1 {
            let mut preceding =
                Self::list_valid_actions_impl(soldiers_count - s, battlefields_count - 1);
            preceding.iter_mut().for_each(|act| (*act).push(s));
            ret.append(&mut preceding);
        }
        ret
    }
}

pub struct Regret {
    regrets: Vec<i64>,
    strategy_sum: Vec<i64>,
}

impl Regret {
    fn new(buf_size: usize) -> Self {
        Self {
            regrets: vec![0; buf_size],
            strategy_sum: vec![0; buf_size],
        }
    }

    /// Returns a vector of probabilities of playing each actions.
    /// For example [2, 3, 5] means that the player plays
    ///   - Rock for 20% of chance
    ///   - Paper for 30% of chance
    ///   - Scissors for 50% of chance
    /// Note that it doesn't need to normalize regrets into [0, 1] values (because WeightedIndex doesn't require normalized values)
    /// but we do so for betterunderstandability.
    fn to_strategy(&self) -> Vec<i64> {
        let normalized_regrets: Vec<i64> = self.regrets.iter().map(|&r| r.max(0)).collect();
        let regret_sum: i64 = normalized_regrets.iter().sum();
        if regret_sum <= 0 {
            return vec![1; self.regrets.len()];
        }
        normalized_regrets
    }

    fn update_strategy_sum(&mut self, strategy: &[i64]) {
        for (i, &s) in strategy.iter().enumerate() {
            self.strategy_sum[i] += s;
        }
    }

    pub fn to_average_strategy(&self) -> Vec<f64> {
        let strategy_sum_total: f64 = self.strategy_sum.iter().sum::<i64>() as f64;
        if strategy_sum_total <= 0.0 {
            return vec![1.0 / self.regrets.len() as f64; self.regrets.len()];
        }
        let mut strategy = vec![0.0; self.regrets.len()];
        for (i, &s) in self.strategy_sum.iter().enumerate() {
            strategy[i] = s as f64 / strategy_sum_total;
        }
        strategy
    }

    pub fn get_action(
        &self,
        rng: &mut impl Rng,
        strategy: &[i64],
        valid_actions: &[Action],
    ) -> Action {
        let dist = WeightedIndex::new(strategy).unwrap();
        valid_actions[dist.sample(rng)].clone()
    }

    fn update_regret(
        &mut self,
        my_action: &Action,
        opponent_action: &Action,
        valid_actions: &[Action],
    ) {
        let payoff = calc_payoff(my_action, opponent_action);
        for action in valid_actions {
            let diff = calc_payoff(action, opponent_action) - payoff;
            self.regrets[action.index] += diff;
        }
    }
}

pub fn calc_payoff(a: &Action, b: &Action) -> i64 {
    assert!(a.assignments.len() == b.assignments.len());
    let mut claimed = 0;
    for i in 0..a.assignments.len() {
        claimed += match a.assignments[i].cmp(&b.assignments[i]) {
            Ordering::Less => -1,
            Ordering::Greater => 1,
            Ordering::Equal => 0,
        };
    }
    match claimed.cmp(&0) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}
