use crate::games::GameState;

pub struct Node<G>
where
    G: GameState,
{
    //info_set: S::InfoSet,
    pub regret_sum: Vec<f64>,
    pub strategy_sum: Vec<f64>,

    pub actions: Vec<G::Action>,
}

impl<G> Node<G>
where
    G: GameState,
{
    pub fn new(actions: Vec<G::Action> /*, info_set: S::InfoSet*/) -> Self {
        let act_len = actions.len();
        Self {
            //info_set,
            actions,

            regret_sum: vec![0.0; act_len],
            strategy_sum: vec![0.0; act_len],
        }
    }

    pub fn regret_matching(&self) -> Vec<f64> {
        let mut sum = 0.0;
        for act_regret_sum in &self.regret_sum {
            sum += act_regret_sum.max(0.0);
        }
        if sum <= 0.0 {
            let s = 1.0 / self.regret_sum.len() as f64;
            return vec![s; self.regret_sum.len()];
        }
        let mut strategy = vec![0.0; self.regret_sum.len()];
        for (i, act_regret_sum) in self.regret_sum.iter().enumerate() {
            strategy[i] = act_regret_sum.max(0.0) / sum;
        }
        strategy
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
