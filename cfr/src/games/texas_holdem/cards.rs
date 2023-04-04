use more_asserts::{assert_gt, debug_assert_ge, debug_assert_lt};
use rand::Rng;

use crate::games::texas_holdem::index_to_rank;

use super::{rank_to_index, Card, SUITS};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cards {
    count: usize,
    bit_fields: u64,
}

impl From<&Vec<Card>> for Cards {
    fn from(value: &Vec<Card>) -> Self {
        let mut cards = Cards::new();
        for card in value {
            cards.push(card);
        }

        cards
    }
}

impl Cards {
    const MASK_ALL: u64 = Self::gen_mask_all();

    const fn gen_mask_all() -> u64 {
        let mut v = 0;
        let mut i = 0;
        while i < 52 {
            let bit = 1 << i;
            v |= bit;
            i += 1;
        }
        v
    }

    pub fn new() -> Self {
        Cards {
            count: 0,
            bit_fields: 0,
        }
    }

    pub fn new_all() -> Self {
        Cards {
            count: 4 * 13,
            bit_fields: Self::MASK_ALL,
        }
    }

    pub fn push(&mut self, card: &Card) {
        let index = Self::card_index(card);
        let bit = 1 << index;
        debug_assert_ge!(index, 0);
        debug_assert_lt!(index, 52);

        assert_eq!(0, self.bit_fields & bit);
        self.bit_fields |= bit;
        self.count += 1;
    }

    #[inline]
    fn pop_by_index(&mut self, index: usize) {
        let bit = 1 << index;
        debug_assert_ge!(index, 0);
        debug_assert_lt!(index, 52);

        assert_eq!(bit, self.bit_fields & bit);
        self.bit_fields ^= bit;
        self.count -= 1;
    }

    pub fn pop(&mut self, card: &Card) {
        let index = Self::card_index(card);
        self.pop_by_index(index);
    }

    pub fn sample_card<R: Rng>(&mut self, rng: &mut R) -> Card {
        assert_gt!(self.count, 0);
        loop {
            let index = rng.gen_range(0..52);
            let bit = 1 << index;
            if self.bit_fields & bit == bit {
                self.pop_by_index(index);
                return Self::index_to_card(index);
            }
        }
    }

    pub fn contains(&self, card: &Card) -> bool {
        let index = Self::card_index(card);
        let bit = 1 << index;
        debug_assert_ge!(index, 0);
        debug_assert_lt!(index, 52);

        self.bit_fields & bit == bit
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn card_index(card: &Card) -> usize {
        let r = rank_to_index(card.rank);
        let s = (card.suit as usize) * 13;
        r + s
    }

    pub fn index_to_card(index: usize) -> Card {
        debug_assert_ge!(index, 0);
        debug_assert_lt!(index, 52);

        let rank = index_to_rank(index % 13);
        let suit = SUITS[index / 13];
        Card {
            rank,
            suit,
        }
    }

    pub fn to_vec(&self) -> Vec<Card> {
        let mut v = vec![];
        for i in 0..52 {
            if self.bit_fields & (1 << i) != 0 {
                v.push(Self::index_to_card(i));
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {

    use crate::games::texas_holdem::list_all_cards;

    use super::*;

    #[test]
    fn test_add_all_cards() {
        let all_cards = list_all_cards();
        let mut cards = Cards::new();

        assert_eq!(0, cards.len());
        for c in &all_cards {
            cards.push(c);
        }

        assert_eq!(52, all_cards.len());
        assert_eq!(52, cards.len());
    }

    #[test]
    fn test_pop_all_cards() {
        let all_cards = list_all_cards();
        let mut cards = Cards::new_all();

        assert_eq!(52, cards.len());
        for c in &all_cards {
            cards.pop(c);
        }

        assert_eq!(52, all_cards.len());
        assert_eq!(0, cards.len());
    }
}
