use std::fmt::Display;

use log::info;
use rand::{
    distributions::WeightedIndex,
    prelude::Distribution,
    Rng,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Rock,
    Paper,
    Scissors,
}

impl Action {
    const VALUES: [Action; 3] = [Action::Rock, Action::Paper, Action::Scissors];
}

pub struct Regret {
    regrets: Vec<i32>,
    strategy_sum: Vec<i32>,
}

impl Regret {
    pub fn new() -> Self {
        Self {
            regrets: vec![0; Action::VALUES.len()],
            strategy_sum: vec![0; Action::VALUES.len()],
        }
    }

    /// Returns a vector of probabilities of playing each actions.
    /// For example [2, 3, 5] means that the player plays
    ///   - Rock for 20% of chance
    ///   - Paper for 30% of chance
    ///   - Scissors for 50% of chance
    /// Note that it doesn't need to normalize regrets into [0, 1] values (because WeightedIndex doesn't require normalized values)
    /// but we do so for betterunderstandability.
    pub fn to_strategy(&self) -> Vec<i32> {
        let normalized_regrets: Vec<i32> = self.regrets.iter().map(|&r| r.max(0)).collect();
        let regret_sum: i32 = normalized_regrets.iter().sum();
        if regret_sum <= 0 {
            return vec![1; self.regrets.len()];
        }
        normalized_regrets
    }

    pub fn update_strategy_sum(&mut self, strategy: &[i32]) {
        for (i, &s) in strategy.iter().enumerate() {
            self.strategy_sum[i] += s;
        }
    }

    pub fn to_average_strategy(&self) -> Vec<f64> {
        let strategy_sum_total: f64 = self.strategy_sum.iter().sum::<i32>() as f64;
        if strategy_sum_total <= 0.0 {
            return vec![1.0 / Action::VALUES.len() as f64; Action::VALUES.len()];
        }
        let mut strategy = vec![0.0; Action::VALUES.len()];
        for (i, &s) in self.strategy_sum.iter().enumerate() {
            strategy[i] = s as f64 / strategy_sum_total;
        }
        strategy
    }

    pub fn accumulate_regret(&mut self, action: Action, regret: i32) {
        self.regrets[action as usize] += regret;
    }

    pub fn update_regret(&mut self, my_action: Action, opponent_action: Action) {
        let payoff = calc_payoff(my_action, opponent_action).0;
        for (i, &action) in Action::VALUES.iter().enumerate() {
            let diff = calc_payoff(action, opponent_action).0 - payoff;
            self.regrets[i] += diff;
        }
    }
}

impl Display for Regret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Regret[")?;
        for r in &self.regrets {
            write!(f, "{:.05}, ", r)?;
        }
        writeln!(f, "]")?;
        write!(f, "Strategy[")?;
        for s in &self.to_strategy() {
            write!(f, "{:.05}, ", s)?;
        }
        writeln!(f, "]")?;
        write!(f, "Avg-Strategy[")?;
        for s in &self.to_average_strategy() {
            write!(f, "{:.05}, ", s)?;
        }
        writeln!(f, "]")?;

        Ok(())
    }
}

impl Default for Regret {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_action(rng: &mut impl Rng, strategy: &[i32]) -> Action {
    let dist = WeightedIndex::new(strategy).unwrap();
    Action::VALUES[dist.sample(rng)]
}

pub fn calc_payoff(a: Action, b: Action) -> (i32, i32) {
    if a == b {
        return (0, 0);
    }
    match a {
        Action::Rock => {
            if b == Action::Scissors {
                (1, -1)
            } else {
                (-1, 1)
            }
        }
        Action::Paper => {
            if b == Action::Rock {
                (1, -1)
            } else {
                (-1, 1)
            }
        }
        Action::Scissors => {
            if b == Action::Paper {
                (1, -1)
            } else {
                (-1, 1)
            }
        }
    }
}

pub fn train(iterations: u32) {
    let mut rng = rand::thread_rng();
    let mut player_regret = Regret::new();
    let mut opponent_regret = Regret::new();
    for i in 0..iterations {
        // Get random action according to mixed-strategy distribution
        let player_strategy = player_regret.to_strategy();
        let opponent_strategy = opponent_regret.to_strategy();

        player_regret.update_strategy_sum(&player_strategy);
        opponent_regret.update_strategy_sum(&opponent_strategy);

        let player_action = get_action(&mut rng, &player_strategy);
        let opponent_action = get_action(&mut rng, &opponent_strategy);

        player_regret.update_regret(player_action, opponent_action);
        opponent_regret.update_regret(opponent_action, player_action);

        if i % 1000 == 0 {
            info!("Player: {}", player_regret);
        }
    }
    info!("Player: {}", player_regret);
}
