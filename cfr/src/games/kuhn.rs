use std::fmt::Display;

use itertools::Itertools;

use super::{
    PlayerId,
    State,
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

impl State for KuhnState {
    type InfoSet = KuhnInfoSet;
    type Action = KuhnAction;

    fn new_root() -> Self {
        Self {
            next_player_id: PlayerId::Chance,
            actions: [None, None],
            cards: [Card::Jack, Card::Jack],
            pot: 2, // ante
        }
    }

    fn to_info_set(&self) -> Self::InfoSet {
        self.into()
    }

    fn list_legal_actions(&self) -> Vec<KuhnAction> {
        vec![KuhnAction::Pass, KuhnAction::Bet]
    }

    fn list_legal_chance_actions(&self) -> Vec<(KuhnAction, f64)> {
        let cards = [Card::Jack, Card::Queen, Card::King];
        let mut v = vec![];
        let pairs = cards.iter().permutations(2).collect_vec();
        let prob = 1.0 / (pairs.len() as f64);
        for s in pairs {
            v.push((KuhnAction::ChanceDealCards([*s[0], *s[1]]), prob));
        }
        v
    }

    fn with_action(&self, action: KuhnAction) -> Self {
        let mut next = self.clone();
        match action {
            KuhnAction::Pass => {
                next.actions[self.next_player_id.index()] = Some(action);
                next.next_player_id = self.next_player_id.opponent();
            }
            KuhnAction::Bet => {
                next.actions[self.next_player_id.index()] = Some(action);
                next.next_player_id = self.next_player_id.opponent();
                next.pot += 1;
            }
            KuhnAction::ChanceDealCards(cards) => {
                next.next_player_id = PlayerId::Player(0);
                next.cards = cards;
            }
        }
        next
    }

    fn is_terminal(&self) -> bool {
        if self.next_player_id == PlayerId::Chance {
            return false;
        }
        if self.actions[self.next_player_id.index()] == Some(KuhnAction::Bet)
            && self.actions[self.next_player_id.opponent().index()] == Some(KuhnAction::Pass)
        {
            // opponent folded
            return true;
        }
        self.actions.iter().all(|a| *a == Some(KuhnAction::Pass))
            || self.actions.iter().all(|a| *a == Some(KuhnAction::Bet))
    }

    fn get_payouts(&self) -> [f64; 2] {
        if self.actions[0] == Some(KuhnAction::Bet) && self.actions[1] == Some(KuhnAction::Pass) {
            // player 1 folded.
            return [1.0, -1.0];
        }

        let win = self.cards[0] > self.cards[1];
        match (self.actions[0], self.actions[1]) {
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

    fn get_node_player_id(&self) -> super::PlayerId {
        self.next_player_id
    }
}
