use rand::Rng;

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

pub trait State: Clone + std::fmt::Debug {
    type InfoSet: Clone
        + std::fmt::Display
        + std::hash::Hash
        + std::cmp::Eq
        + std::cmp::PartialOrd
        + std::cmp::Ord;
    type Action: Copy + std::fmt::Display + std::fmt::Debug + std::cmp::Eq + std::hash::Hash;

    fn new_root<R: Rng>(rng: &mut R) -> Self;
    fn new_root2() -> Self {
        todo!();
    }

    // TODO: Implement it with a chance node. Remove it.
    fn list_possible_root_states() -> Vec<Self> {
        todo!();
    }

    fn to_info_set(&self) -> Self::InfoSet;

    fn is_terminal(&self) -> bool;

    // TODO: Make it vector or scalar (but with an argument player_id)
    fn get_payouts(&self) -> [f64; 2];

    fn get_node_player_id(&self) -> PlayerId;

    fn with_action(&self, action: Self::Action) -> Self;

    fn list_legal_actions(&self) -> Vec<Self::Action>;
    fn list_legal_chance_actions(&self) -> Vec<(Self::Action, f64)> {
        todo!();
    }
}
