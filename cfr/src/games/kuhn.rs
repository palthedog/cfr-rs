use std::fmt::Display;

use itertools::Itertools;

use super::{
    Game,
    PlayerId,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Card {
    Jack = 0,
    Queen = 1,
    King = 2,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum KuhnAction {
    Pass,
    Bet,

    // Chance actions
    ChanceDealCards([Card; 2]),
}

impl Display for KuhnAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KuhnInfoSet {
    pub player_id: PlayerId,
    pub card: Card,
    pub actions: [Option<KuhnAction>; 2],
}

impl From<&KuhnState> for KuhnInfoSet {
    fn from(state: &KuhnState) -> Self {
        KuhnInfoSet {
            player_id: state.next_player_id,
            card: state.cards[state.next_player_id.index()],
            actions: state.actions,
        }
    }
}

impl Display for KuhnInfoSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}({:5}): ", self.player_id.index(), format!("{:?}", self.card))?;
        write!(
            f,
            "[{:11},{:11}]",
            format!("{:?}", self.actions[0]),
            format!("{:?}", self.actions[1])
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KuhnState {
    pub next_player_id: PlayerId,
    pub actions: [Option<KuhnAction>; 2],
    pub cards: [Card; 2],
    pub pot: i32,
}

pub struct Kuhn {}

impl Kuhn {
    pub fn new() -> Self {
        Self {}
    }
}

impl Game for Kuhn {
    type State = KuhnState;
    type InfoSet = KuhnInfoSet;
    type Action = KuhnAction;

    fn new_root(&self) -> Self::State {
        Self::State {
            next_player_id: PlayerId::Chance,
            actions: [None, None],
            cards: [Card::Jack, Card::Jack],
            pot: 2, // ante
        }
    }

    fn to_info_set(&self, state: &Self::State) -> Self::InfoSet {
        state.into()
    }

    fn list_legal_actions(&self, _state: &Self::State) -> Vec<KuhnAction> {
        vec![KuhnAction::Pass, KuhnAction::Bet]
    }

    fn list_legal_chance_actions(&self, _state: &Self::State) -> Vec<(KuhnAction, f64)> {
        let cards = [Card::Jack, Card::Queen, Card::King];
        let mut v = vec![];
        let pairs = cards.iter().permutations(2).collect_vec();
        let prob = 1.0 / (pairs.len() as f64);
        for s in pairs {
            v.push((KuhnAction::ChanceDealCards([*s[0], *s[1]]), prob));
        }
        v
    }

    fn with_action(&self, state: &Self::State, action: KuhnAction) -> Self::State {
        let mut next = state.clone();
        match action {
            KuhnAction::Pass => {
                next.actions[state.next_player_id.index()] = Some(action);
                next.next_player_id = state.next_player_id.opponent();
            }
            KuhnAction::Bet => {
                next.actions[state.next_player_id.index()] = Some(action);
                next.next_player_id = state.next_player_id.opponent();
                next.pot += 1;
            }
            KuhnAction::ChanceDealCards(cards) => {
                next.next_player_id = PlayerId::Player(0);
                next.cards = cards;
            }
        }
        next
    }

    fn is_terminal(&self, state: &Self::State) -> bool {
        if state.next_player_id == PlayerId::Chance {
            return false;
        }
        if state.actions[state.next_player_id.index()] == Some(KuhnAction::Bet)
            && state.actions[state.next_player_id.opponent().index()] == Some(KuhnAction::Pass)
        {
            // opponent folded
            return true;
        }
        state.actions.iter().all(|a| *a == Some(KuhnAction::Pass))
            || state.actions.iter().all(|a| *a == Some(KuhnAction::Bet))
    }

    fn get_payouts(&self, state: &Self::State) -> [f64; 2] {
        if state.actions[0] == Some(KuhnAction::Bet) && state.actions[1] == Some(KuhnAction::Pass) {
            // player 1 folded.
            return [1.0, -1.0];
        }

        let win = state.cards[0] > state.cards[1];
        match (state.actions[0], state.actions[1]) {
            (Some(KuhnAction::Pass), Some(KuhnAction::Bet)) => [-1.0, 1.0], // ante
            (Some(KuhnAction::Bet), Some(KuhnAction::Pass)) => [1.0, -1.0],
            (Some(KuhnAction::Pass), Some(KuhnAction::Pass)) => {
                if win {
                    [1.0, -1.0]
                } else {
                    [-1.0, 1.0]
                }
            }
            (Some(KuhnAction::Bet), Some(KuhnAction::Bet)) => {
                if win {
                    [2.0, -2.0]
                } else {
                    [-2.0, 2.0]
                }
            }
            _ => panic!(),
        }
    }

    fn get_node_player_id(&self, state: &Self::State) -> super::PlayerId {
        state.next_player_id
    }
}
