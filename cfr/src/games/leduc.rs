use std::fmt::Display;

use itertools::Itertools;
use log::debug;
use more_asserts::debug_assert_ge;

use super::{
    PlayerId,
    State,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    Jack,
    Queen,
    King,
}

impl Rank {
    pub const COUNT: usize = 3;
    pub const VALUES: [Rank; Self::COUNT] = [Rank::King, Rank::Queen, Rank::Jack];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card {
    pub rank: Rank,
}

impl Card {
    fn get_all() -> Vec<Card> {
        let mut v = vec![];
        for rank in Rank::VALUES {
            // two cards for each rank
            v.push(Card {
                rank,
            });
            v.push(Card {
                rank,
            });
        }
        v
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self.rank {
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
        };
        write!(f, "{}", r)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LeducAction {
    Check,
    Raise,
    Call,
    Fold,

    ChanceDealCards([Card; 2], Card),
}

impl LeducAction {
    const VALUES: [LeducAction; 4] =
        [LeducAction::Check, LeducAction::Raise, LeducAction::Call, LeducAction::Fold];
}

impl Display for LeducAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LeducRound {
    Preflop,
    Flop,

    Folded(PlayerId),
    ShowDown,
}

impl LeducRound {
    fn next(&self) -> LeducRound {
        match self {
            LeducRound::Preflop => LeducRound::Flop,
            LeducRound::Flop => LeducRound::ShowDown,
            LeducRound::ShowDown => panic!(),
            LeducRound::Folded(_) => panic!(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeducInfoSet {
    pub player_id: PlayerId,
    pub round: LeducRound,
    pub hole_card: Card,
    pub community_card: Option<Card>,
    pub actions: Vec<LeducAction>,

    // includes blinds
    pub bets: [i32; 2],
    // Raise count in this round
    pub raise_count: i32,
}

impl From<&LeducState> for LeducInfoSet {
    fn from(state: &LeducState) -> Self {
        let community_card = match state.round {
            LeducRound::Preflop => None,
            _ => state.community_card,
        };

        LeducInfoSet {
            player_id: state.next_player_id,
            round: state.round,
            hole_card: state.hole_cards.unwrap()[state.next_player_id.index()],
            community_card,
            actions: state.actions.clone(),
            bets: state.bets,
            raise_count: state.raise_count,
        }
    }
}

impl Display for LeducInfoSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}({})", self.player_id.index(), self.hole_card)?;
        if self.round != LeducRound::Preflop {
            write!(f, " {}", self.community_card.unwrap())?;
        }
        write!(f, ": [")?;
        for act in self.actions.iter() {
            write!(f, "{:?}, ", act)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeducState {
    pub next_player_id: PlayerId,

    pub round: LeducRound,

    pub actions: Vec<LeducAction>,
    pub hole_cards: Option<[Card; 2]>,
    pub community_card: Option<Card>,

    // includes blinds
    pub bets: [i32; 2],
    // Raise count in this round
    pub raise_count: i32,
}

impl LeducState {
    fn is_valid_action(&self, action: LeducAction) -> bool {
        match action {
            LeducAction::Check => self.bets[0] == self.bets[1],
            LeducAction::Raise => self.raise_count < 2,
            LeducAction::Call => {
                let p = self.next_player_id.index();
                let o = self.next_player_id.opponent().index();
                self.bets[p] < self.bets[o]
            }
            LeducAction::Fold => {
                let p = self.next_player_id.index();
                let o = self.next_player_id.opponent().index();
                self.bets[p] < self.bets[o]
            }
            LeducAction::ChanceDealCards(_, _) => {
                self.round == LeducRound::Preflop && self.hole_cards.is_none()
            }
        }
    }

    fn update(&mut self, action: LeducAction) {
        debug_assert!(self.is_valid_action(action), "invalid action");

        if let LeducAction::ChanceDealCards(hole_cards, community_card) = action {
            self.hole_cards = Some(hole_cards);
            self.community_card = Some(community_card);
            self.next_player_id = PlayerId::Player(0);
            return;
        }

        let mut go_to_next = false;
        match action {
            LeducAction::Check => {
                let p = self.next_player_id.index();
                if p == 1 {
                    // go to next round
                    go_to_next = true;
                }
            }
            LeducAction::Raise => {
                let p = self.next_player_id.index();
                let o = self.next_player_id.opponent().index();
                self.raise_count += 1;
                self.bets[p] = self.bets[o] + self.raise_amount();
            }
            LeducAction::Call => {
                let p = self.next_player_id.index();
                let o = self.next_player_id.opponent().index();
                self.bets[p] = self.bets[o];
                go_to_next = true;
            }
            LeducAction::Fold => {
                self.round = LeducRound::Folded(self.next_player_id);
            }
            LeducAction::ChanceDealCards(_, _) => {
                unreachable!();
            }
        }

        if go_to_next {
            self.round = self.round.next();
            self.raise_count = 0;
            self.next_player_id = PlayerId::Player(0);
        } else {
            self.next_player_id = self.next_player_id.opponent();
        }
        self.actions.push(action);
    }

    fn raise_amount(&self) -> i32 {
        match self.round {
            LeducRound::Preflop => 2,
            LeducRound::Flop => 4,
            LeducRound::ShowDown => panic!(),
            LeducRound::Folded(_) => panic!(),
        }
    }

    fn calc_hand_rank(cards: [Card; 2]) -> u32 {
        let mut cs = cards;
        cs.sort_by(|a, b| b.rank.cmp(&a.rank));
        debug_assert_ge!(cs[0].rank, cs[1].rank);

        // pair? | higher-rank(2 bits) | lower-rank(2 bits)
        let mut ret: u32 = 0;
        if cs[0].rank == cs[1].rank {
            // one pair
            ret = 1;
        }
        ret = (ret << 2) | cs[0].rank as u32;
        ret = (ret << 2) | cs[1].rank as u32;
        ret
    }
}

impl State for LeducState {
    type InfoSet = LeducInfoSet;
    type Action = LeducAction;

    fn new_root() -> Self {
        Self {
            next_player_id: PlayerId::Chance,
            round: LeducRound::Preflop,
            actions: vec![],
            hole_cards: None,
            community_card: None,
            bets: [1, 1],
            raise_count: 0,
        }
    }

    fn to_info_set(&self) -> Self::InfoSet {
        self.into()
    }

    fn is_terminal(&self) -> bool {
        match self.round {
            LeducRound::Preflop => false,
            LeducRound::Flop => false,
            LeducRound::Folded(_) => true,
            LeducRound::ShowDown => true,
        }
    }

    fn get_payouts(&self) -> [f64; 2] {
        debug_assert!(self.is_terminal());

        let loser: usize;
        let winner: usize;
        match self.round {
            LeducRound::Folded(pid) => {
                loser = pid.index();
                winner = pid.opponent().index();
            }
            LeducRound::ShowDown => {
                let p = Self::calc_hand_rank([
                    self.hole_cards.unwrap()[0],
                    self.community_card.unwrap(),
                ]);
                let o = Self::calc_hand_rank([
                    self.hole_cards.unwrap()[1],
                    self.community_card.unwrap(),
                ]);
                if p == o {
                    return [0.0, 0.0];
                }
                if p > o {
                    winner = 0;
                    loser = 1;
                } else {
                    winner = 1;
                    loser = 0;
                }
            }
            LeducRound::Preflop => panic!(),
            LeducRound::Flop => panic!(),
        }

        let mut ret = [0.0, 0.0];
        ret[winner] = self.bets[loser] as f64;
        ret[loser] = -self.bets[loser] as f64;

        debug!(
            "{} v.s {}, {}  acts: {:?}, payouts: {:?}",
            self.hole_cards.unwrap()[0],
            self.hole_cards.unwrap()[1],
            self.community_card.unwrap(),
            self.actions,
            ret
        );

        ret
    }

    fn get_node_player_id(&self) -> super::PlayerId {
        self.next_player_id
    }

    fn with_action(&self, action: LeducAction) -> Self {
        let mut next = self.clone();
        next.update(action);
        next
    }

    fn list_legal_actions(&self) -> Vec<LeducAction> {
        let mut v = vec![];
        for act in LeducAction::VALUES {
            if self.is_valid_action(act) {
                v.push(act);
            }
        }
        v
    }

    fn list_legal_chance_actions(&self) -> Vec<(Self::Action, f64)> {
        assert_eq!(LeducRound::Preflop, self.round);
        let all_cards = Card::get_all();
        let len = count_permutations(all_cards.len(), 3);
        let all_combinations = all_cards.iter().permutations(3);
        let prob = 1.0 / len as f64;
        let mut v = Vec::with_capacity(len);
        for cards in all_combinations {
            let act = LeducAction::ChanceDealCards([*cards[0], *cards[1]], *cards[2]);
            v.push((act, prob));
        }
        v
    }
}

fn count_permutations(n: usize, r: usize) -> usize {
    (n - r + 1..=n).product()
}
