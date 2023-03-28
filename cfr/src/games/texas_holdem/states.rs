use std::{
    cmp,
    fmt::{
        self,
        Display,
    },
};

use log::info;

use super::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub enum Round {
    #[default]
    Preflop,
    Flop,
    Turn,
    River,
}

impl From<usize> for Round {
    fn from(n: usize) -> Round {
        int_to_round(n)
    }
}

pub fn int_to_round(l: usize) -> Round {
    match l {
        0 => Round::Preflop,
        1 => Round::Flop,
        2 => Round::Turn,
        3 => Round::River,
        _ => panic!("Unknown {}", l),
    }
}

pub fn cs_len_to_round(l: usize) -> Round {
    match l {
        0 => Round::Preflop,
        3 => Round::Flop,
        4 => Round::Turn,
        5 => Round::River,
        _ => panic!("Unknown {}", l),
    }
}



impl fmt::Display for Round {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Round::Preflop => "Preflop",
            Round::Flop => "Flop",
            Round::Turn => "Turn",
            Round::River => "River",
        };
        write!(f, "{}", s)
    }
}

impl Round {
    pub fn next(&self) -> Round {
        match *self {
            Round::Preflop => Round::Flop,
            Round::Flop => Round::Turn,
            Round::Turn => Round::River,
            Round::River => {
                panic!("Failed to get the next round because the given round is RIVER.");
            }
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerState {
    pub stack: i32,
    pub bet: i32,
    pub took_action: bool,

    pub folded: bool,
    pub hole_cards: Vec<Card>,
}

impl PlayerState {
    pub fn is_folded(&self) -> bool {
        self.folded
    }

    pub fn is_all_in(&self) -> bool {
        self.bet == self.stack
    }

    pub fn agreed(&self, max_bet: i32) -> bool {
        self.bet == max_bet || self.is_all_in()
    }
}

impl fmt::Display for PlayerState {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "bet: {}, cards: {}", self.bet, cards_to_str(&self.hole_cards))
    }
}

#[allow(clippy::derivable_impls)]
impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState {
            stack: 0,
            bet: 0,
            took_action: false,
            folded: false,
            hole_cards: vec![],
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoundState {
    /// Minimum bet/raise amount. It includes bet size even acted in the previous round.
    /// For example, if
    /// - bb/sb = 100/50
    /// - preflop: sb raised to 200, bb called
    /// - flop: at this point `min_raise_to` would be 300 (min raise amout = 100, bb already bet 200)
    pub min_raise_to: i32,
    pub bet_cnt: i32,
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandState {
    pub next_player: usize,
    pub last_action: Option<Action>,
    pub round: Round,
    pub round_state: RoundState,

    //pub my_position: usize,
    pub community_cards: Vec<Card>,

    pub players: Vec<PlayerState>,
}

impl HandState {
    pub fn get_next_player_state_ref(&self) -> &PlayerState {
        &self.players[self.next_player]
    }

    /// Returns the bet size which
    pub fn max_bet(&self) -> i32 {
        self.players.iter().fold(0, |a, p| cmp::max(a, p.bet))
    }

    pub fn pot(&self) -> i32 {
        self.players.iter().fold(0, |pot, p| pot + p.bet)
    }

    pub fn round_is_finished(&self) -> bool {
        self.everyone_agreed()
    }

    pub fn hand_is_finished(&self, round_is_finished: bool) -> bool {
        if self.round == Round::River && round_is_finished {
            return true;
        }
        if self.everyone_all_in() && self.community_cards.len() == 5 {
            return true;
        }
        self.everyone_folded()
    }

    pub fn everyone_all_in(&self) -> bool {
        for p in self.players.iter().filter(|p| !p.is_folded()) {
            if !p.is_all_in() {
                return false;
            }
        }
        true
    }

    /// Return a number of players who can take actions.
    /// Players can't take action if the player:
    ///   - already folded
    ///   - or already did all-in
    pub fn count_acting_players(&self) -> u32 {
        let mut count = 0;
        for p in self.players.iter() {
            if !p.is_folded() && !p.is_all_in() {
                count += 1;
            }
        }
        count
    }

    pub fn everyone_agreed(&self) -> bool {
        let bet = self.max_bet();
        for p in self.players.iter().filter(|p| !p.is_folded()) {
            if !p.took_action || !p.agreed(bet) {
                return false;
            }
        }
        true
    }

    pub fn everyone_folded(&self) -> bool {
        let mut not_folded = 0;
        for p in &self.players {
            if !p.folded {
                not_folded += 1;
            }
        }
        not_folded == 1
    }

    pub fn calculate_won_pots(&self) -> HandResult {
        assert!(
            self.hand_is_finished(self.round_is_finished())
                || (self.everyone_all_in() && self.community_cards.len() == 5)
        );

        let mut scores = Vec::with_capacity(self.players.len());
        let mut max_score = HandScore::fold();
        for (i, player) in self.players.iter().enumerate() {
            let score;
            if !player.folded {
                score = hands::calc_player_score(self, player);
                info!("  score@{}: {}", i, score);
                max_score = max_score.max(score);
            } else {
                score = HandScore::fold();
                info!("  score@{}: fold", i);
            }
            scores.push(score);
        }

        let mut won_pots = Vec::with_capacity(self.players.len());
        let mut hands = Vec::with_capacity(self.players.len());
        let winner_cnt = scores.iter().filter(|&a| *a == max_score).count();
        let won_amount = self.pot() / winner_cnt as i32;
        info!("  pot: {}, won: {}, winner_cnt: {}", self.pot(), won_amount, winner_cnt);
        for (player, score) in self.players.iter().zip(scores.iter()) {
            let won = if *score == max_score {
                won_amount - player.bet
            } else {
                0 - player.bet
            };
            won_pots.push(won);
            hands.push(*score);
        }
        HandResult {
            won_pots,
            hands,
        }
    }

    pub fn dump(&self) -> String {
        let mut s = String::new();
        s.push_str("State\n");

        s.push_str(&format!("  {}\n", &cards_to_str(&self.community_cards)));
        s.push_str(&format!("  Pot:{}\n", self.pot()));
        for (i, p) in self.players.iter().enumerate() {
            let act = if self.next_player == i {
                '*'
            } else {
                ' '
            };
            s.push_str(&format!("  {}Player {}: {}\n", act, i, p));
        }
        s
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Fold,
    Call,
    RaiseTo(i32),
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Fold => write!(f, "Fold")?,
            Action::Call => write!(f, "Call")?,
            Action::RaiseTo(amount) => write!(f, "RaiseTo({})", amount)?,
        }
        Ok(())
    }
}
