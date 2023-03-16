use std::fmt;

use super::{
    dealer::{
        Dealer,
        HandResult,
        UpdateResult::*,
    },
    deck::Deck,
    *,
};

use log::info;
use rand::Rng;

/*
pub struct Game {
    pub dealer: Dealer,
    pub hand_state: HandState,
    dealer_pos: usize,
    deck: Deck,
    players: Vec<Box<dyn Player>>,
}

impl Game {
    pub fn new(dealer: Dealer, deck: Deck, players: Vec<Box<dyn Player>>) -> Game {
        Game {
            deck,
            dealer,
            dealer_pos: 0,
            hand_state: HandState::default(),
            players,
        }
    }

    pub fn play_hand<R: Rng>(&mut self, rng: &mut R) -> HandResult {
        info!("play_hand: dealer: {}", self.dealer_pos);
        self.deck.shuffle_first_n(rng, 9);
        self.hand_state = HandState::default();
        self.dealer.init_round_and_deal_cards(&mut self.hand_state, &mut self.deck, Round::Preflop);
        loop {
            let next = (self.dealer_pos + self.hand_state.next_player) % self.players.len();
            let act = self.players[next].next(&self.hand_state);
            if let Some(result) = self.step(act) {
                let result = result.reflect_dealer_pos(self.dealer_pos);
                self.dealer_pos = (self.dealer_pos + 1) % self.players.len();
                return result;
            }
        }
    }

    fn step(&mut self, a: Action) -> Option<HandResult> {
        let next = self.dealer.update(&mut self.hand_state, a);
        match next {
            Keep => None,
            NextRound(next_round) => {
                self.dealer.init_round_and_deal_cards(
                    &mut self.hand_state,
                    &mut self.deck,
                    next_round,
                );
                None
            }
            AllIn => Some(self.dealer.handle_all_in(&mut self.hand_state, &mut self.deck)),
            // Caller must call .init later.
            NextHand => Some(self.dealer.calculate_won_pots(&self.hand_state)),
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.hand_state.dump())
    }
}
*/
