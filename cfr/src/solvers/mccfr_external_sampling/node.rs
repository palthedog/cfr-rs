use crate::games::Game;

pub struct Node<G>
where
    G: Game,
{
    pub regret_sum: Vec<f64>,
    pub strategy_sum: Vec<f64>,
    pub strategy: Vec<f64>,

    pub actions: Vec<G::Action>,
}

impl<G> Node<G>
where
    G: Game,
{
    pub fn new(actions: Vec<G::Action>) -> Self {
        let act_len = actions.len();
        Self {
            actions,

            regret_sum: vec![0.0; act_len],
            strategy_sum: vec![0.0; act_len],
            strategy: vec![0.0; act_len],
        }
    }

    pub fn update_strategy_sum(&mut self) {
        for (i, act_prob) in self.strategy.iter().enumerate() {
            self.strategy_sum[i] += act_prob;
        }
    }

    pub fn regret_matching(&mut self) {
        let mut sum = 0.0;
        for (i, act_regret_sum) in self.regret_sum.iter().enumerate() {
            let pos_regret = act_regret_sum.max(0.0);
            sum += pos_regret;
            self.strategy[i] = pos_regret;
        }
        if sum <= 0.0 {
            let s = 1.0 / self.regret_sum.len() as f64;
            self.strategy.fill(s);
        } else {
            for (i, act_regret_sum) in self.regret_sum.iter().enumerate() {
                self.strategy[i] = act_regret_sum.max(0.0) / sum;
            }
        }
    }

    #[inline]
    pub fn get_strategy(&self) -> &[f64] {
        &self.strategy
    }

    pub fn to_average_strategy(&self) -> Vec<f64> {
        let normalizing_sum: f64 = self.strategy_sum.iter().sum();
        if normalizing_sum == 0.0 {
            let actions_len = self.strategy_sum.len();
            return vec![1.0 / actions_len as f64; actions_len];
        }
        self.strategy_sum.iter().map(|s| s / normalizing_sum).collect()
    }

    #[inline]
    pub fn get_actions(&self) -> &[G::Action] {
        &self.actions
    }
}
