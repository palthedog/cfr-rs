use rand::Rng;

pub mod dudo;

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
            PlayerId::Player(_) => todo!("REMOVE this method"),
            PlayerId::Chance => panic!(),
        }
    }
}

pub trait InfoSet: std::fmt::Display + std::hash::Hash + std::cmp::Eq + Clone {
    type Action: std::fmt::Display + std::fmt::Debug + Copy;

    fn list_legal_actions(&self) -> Vec<Self::Action>;
}

pub trait State: Clone + std::fmt::Debug {
    type InfoSet: InfoSet;

    fn new_root<R: Rng>(rng: &mut R) -> Self;

    fn to_info_set(&self) -> Self::InfoSet;

    fn is_terminal(&self) -> bool;

    // TODO: Make it vector or scalar (but with an argument player_id)
    fn get_payouts(&self) -> [f64; 2];

    fn get_node_player_id(&self) -> PlayerId;

    fn with_action(&self, action: <<Self as State>::InfoSet as InfoSet>::Action) -> Self;
}
