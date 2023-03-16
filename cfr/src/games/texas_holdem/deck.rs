use rand::Rng;

use super::*;
use std::fmt;

pub struct Deck {
    pos: usize,
    cards: Vec<Card>,
}

pub fn all_cards() -> Vec<Card> {
    let mut cards = Vec::with_capacity(13 * 4);
    for suit in &SUITS {
        for rank in 2..15 {
            cards.push(Card {
                rank,
                suit: *suit,
            });
        }
    }
    cards
}

impl Default for Deck {
    #[allow(dead_code)]
    fn default() -> Deck {
        Deck {
            pos: 0,
            cards: all_cards(),
        }
    }
}

impl Deck {
    pub fn empty() -> Deck {
        Deck {
            pos: 0,
            cards: vec![],
        }
    }

    pub fn new_without(cs: &[Card]) -> Deck {
        let mut cards: Vec<Card> = Vec::with_capacity(13 * 4 - cs.len());
        for new in &all_cards() {
            if !cs.contains(new) {
                cards.push(*new);
            }
        }
        Deck {
            pos: 0,
            cards,
        }
    }

    // Puts the given cards on the top of the deck.
    pub fn cheat(top: &[Card]) -> Deck {
        let mut cards: Vec<Card> = Vec::with_capacity(13 * 4);
        for card in top {
            cards.push(*card);
        }
        for new in &all_cards() {
            if !top.contains(new) {
                cards.push(*new);
            }
        }
        Deck {
            pos: 0,
            cards,
        }
    }

    /* We shouldn't use it. Instead always use shuffle_first_n(N) where N = the number of
       cards which will be used in the hand.
    #[allow(dead_code)]
    pub fn shuffle<T: Rng>(&mut self, rng: &mut T) {
        rng.shuffle(&mut self.cards);
        self.pos = 0;
    }
    */

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    // Shuffle the deck for next n draws.
    pub fn shuffle_next_n<T: Rng>(&mut self, rng: &mut T, n: usize) {
        let last = self.cards.len();
        assert!(n < last);
        let mut i = self.pos;
        let e = self.pos + n;
        while i < e {
            let frm = rng.gen_range(i..last);
            if i != frm {
                self.cards.swap(i, frm);
            }
            i += 1;
        }
    }

    pub fn shuffle_first_n<T: Rng>(&mut self, rng: &mut T, n: usize) {
        self.pos = 0;
        self.shuffle_next_n(rng, n);
    }

    pub fn draw(&mut self) -> Card {
        let card = self.cards[self.pos];
        self.pos += 1;
        card
    }

    pub fn draw_n(&mut self, n: usize) -> &[Card] {
        let cards = &self.cards[self.pos..self.pos + n];
        self.pos += n;
        cards
    }
}

impl fmt::Debug for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Deck( pos: {:?}, cards: {:?})",
            self.pos,
            self.cards.iter().take(9).collect::<Vec<_>>()
        )
    }
}
