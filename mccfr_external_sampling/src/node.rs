use cfr::games::State;

pub struct Node<S>
where
    S: State,
{
    //info_set: S::InfoSet,
    pub regret_sum: Vec<f64>,
    pub strategy_sum: Vec<f64>,

    pub actions: Vec<S::Action>,
}

impl<S> Node<S>
where
    S: State,
{
    pub fn new(actions: Vec<S::Action> /*, info_set: S::InfoSet*/) -> Self {
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

    #[inline]
    pub fn get_actions(&self) -> &[S::Action] {
        &self.actions
    }
}
