use rand::Rng;
use rand_distr::{
    Distribution,
    WeightedIndex,
};

pub mod dudo;
pub mod kuhn;
pub mod leduc;

// TODO: Make it something like
// ```
// type PlayerId = usize;
// enum PlayerType {
//   Chance,
//   Player(PlayerId)
// }
// ```
// So that we can use raw PlayerId where there is no chance to have ChanceNode.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerId {
    Chance,
    Player(usize),
}

impl PlayerId {
    pub fn index(&self) -> usize {
        match self {
            PlayerId::Player(i) => *i,
            PlayerId::Chance => panic!(),
        }
    }

    pub fn opponent(&self) -> PlayerId {
        match self {
            PlayerId::Player(0) => PlayerId::Player(1),
            PlayerId::Player(1) => PlayerId::Player(0),
            PlayerId::Player(_) => todo!("REMOVE this method to support more than 2 players."),
            PlayerId::Chance => panic!(),
        }
    }
}

pub trait Game {
    type State: Clone
        + std::fmt::Debug
        + std::hash::Hash
        + std::cmp::Eq
        + std::cmp::PartialOrd
        + std::cmp::Ord;
    type InfoSet: Clone
        + std::fmt::Display
        + std::fmt::Debug
        + std::hash::Hash
        + std::cmp::Eq
        + std::cmp::PartialOrd
        + std::cmp::Ord;
    type Action: Copy + std::fmt::Display + std::fmt::Debug + std::cmp::Eq + std::hash::Hash;

    fn new_root(&self) -> Self::State;

    fn to_info_set(&self, state: &Self::State) -> Self::InfoSet;

    fn is_terminal(&self, state: &Self::State) -> bool;

    // TODO: Make it vector or scalar (but with an argument player_id)
    fn get_payouts(&self, state: &Self::State) -> [f64; 2];

    fn get_node_player_id(&self, state: &Self::State) -> PlayerId;

    fn with_action(&self, state: &Self::State, action: Self::Action) -> Self::State;
    fn list_legal_actions(&self, state: &Self::State) -> Vec<Self::Action>;
    fn list_legal_chance_actions(&self, _state: &Self::State) -> Vec<(Self::Action, f64)> {
        todo!();
    }
    fn sample_chance_action<R: Rng>(&self, rng: &mut R, state: &Self::State) -> Self::Action {
        let actions = self.list_legal_chance_actions(state);

        let dist = WeightedIndex::new(actions.iter().map(|p| p.1)).unwrap_or_else(|e| {
            panic!("Invalid weights: e: {} probs: {:?}", e, actions);
        });
        let index = dist.sample(rng);
        actions[index].0
    }
}
