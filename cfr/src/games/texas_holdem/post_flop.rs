use std::{
    fs::File,
    io::{
        BufRead,
        BufReader,
    },
    path::PathBuf,
};

use card::list_all_cards;
use itertools::Itertools;
use log::info;

use super::{
    card,
    Card,
    Dealer,
    HandState,
};

pub struct PreflopStrategy {
    /// Preflop strategy is stored as `strategy[rank_y][rank_x]`.
    /// Just like as popular hand range tables
    /// bet/call probabilities for suited hands are stored in upper-right area (x > y) and
    /// ones for unsuited hands are stored in lower-left area (x <= y).
    /// For example:
    ///   strategy[0][0]: AA
    ///   strategy[0][1]: AKs
    ///   strategy[0][2]: AQs
    ///   strategy[1][0]: AKo
    ///   strategy[1][1]: KK
    ///   strategy[12][12]: 22
    strategy: [[f64; card::RANK_COUNT]; card::RANK_COUNT],
}

impl PreflopStrategy {
    pub fn from_config(path: PathBuf) -> Self {
        let mut strategy = Self {
            strategy: [[0.0; card::RANK_COUNT]; card::RANK_COUNT],
        };
        info!("Start reading a config file: {}", path.display());
        let f = File::open(&path)
            .unwrap_or_else(|e| panic!("Failed to open config file {}: {}", path.display(), e));
        let reader = BufReader::new(f);

        let mut y = 0;
        let mut sum = 0.0;
        for l in reader.lines() {
            let line = l.unwrap_or_else(|e| panic!("Failed to read a line: {}", e));
            let mut x = 0;
            for cell in line.split(',') {
                let cell = cell.trim();
                let val: f64 = cell
                    .parse()
                    .unwrap_or_else(|e| panic!("Failed to parse '{}' as f64: {}", cell, e));
                assert!(
                    val >= 0.0,
                    "All Bet/Call probabilities in {} must be either zero or positive but {}",
                    path.display(),
                    val
                );
                sum += val;
                strategy.strategy[y][x] = val;
                x += 1;
            }
            assert!(x == card::RANK_COUNT);
            y += 1;
        }
        assert!(y == card::RANK_COUNT);

        strategy
    }

    pub fn from_array(strategy: [[f64; card::RANK_COUNT]; card::RANK_COUNT]) -> Self {
        let mut sum = 0.0;
        for y in 0..card::RANK_COUNT {
            for x in 0..card::RANK_COUNT {
                let val = strategy[y][x];
                assert!(
                    val >= 0.0,
                    "All Bet/Call probabilities must be either zero or positive but {} at x: {}, y: {}",
                    val,
                    x,
                    y
                );
                sum += val;
            }
        }
        Self {
            strategy,
        }
    }

    pub fn get(&self, card0: &Card, card1: &Card) -> f64 {
        self.get_from_ranks(card0.rank, card1.rank, card0.suit == card1.suit)
    }

    pub fn get_from_ref_slice(&self, cards: &[&Card]) -> f64 {
        debug_assert_eq!(2, cards.len());
        unsafe { self.get(cards.get_unchecked(0), cards.get_unchecked(1)) }
    }

    fn to_indices(&self, rank0: card::Rank, rank1: card::Rank, suited: bool) -> (usize, usize) {
        let stronger = rank0.max(rank1);
        let weaker = rank0.min(rank1);
        if suited {
            (card::rank_to_index(weaker), card::rank_to_index(stronger))
        } else {
            (card::rank_to_index(stronger), card::rank_to_index(weaker))
        }
    }

    pub fn get_from_ranks(&self, rank0: card::Rank, rank1: card::Rank, suited: bool) -> f64 {
        let (y, x) = self.to_indices(rank0, rank1, suited);
        self.strategy[y][x]
    }

    pub fn set(&mut self, card0: &Card, card1: &Card, prob: f64) {
        let (y, x) = self.to_indices(card0.rank, card1.rank, card0.suit == card1.suit);
        self.strategy[y][x] = prob;
    }
}

pub struct TexasHoldemPostFlopGame {
    pub dealer: Dealer,
    pub hand_state: HandState,
}

impl TexasHoldemPostFlopGame {}

/// Calculate reach probabilities of postflop round for each possible opponent's hole cards.
/// It presumes
///   - the player always call/bet
///   - the opponent player calls/bets with the given `opponent_strategy`
///   - both players don't raise (i.e. it doesn't consier 3-bets)
///
/// According to the Bayes' theorem, if the opponent bets/calls, we can calculate probabilities of
/// the opponent player's hole cards by
///    P(h | B) = P(B|h) * P(h) / P(B)
/// where
///  - P(h) is probability of getting the given hole cards
///    - P(h) = 1 / C(52, 2) = 1 / 1326  (constant value)
///  - P(B) is a probability of playing calls/bets
///    - P(B) = (P(h0) * P(B | h0) + P(h1) * P(B | h1) + ... + P(h1325) * P(B | h1325))
///  - P(B | h) is a conditional probability
///    - P(B|h) = `opponent_strategy.get(h)`
/// We can simplify P(h) / P(B) because all P(h) is constant value.
/// P(h) / P(B) = P(h) / (P(h) * P(B | h0) + P(h) * P(B | h1) + ... + P(h) * P(B | h1325))
///             = 1 / (P(B | h0) + P(B | h1) + ... + P(B | h1325))
/// P(h | B) = P(B|h) / (P(B | h0) + P(B | h1) + ... + P(B | h1325))
pub fn preflop_strategy_to_post_flop_reach_probabilities(
    opponent_strategy: &PreflopStrategy,
) -> Vec<([Card; 2], f64)> {
    // Compute P(h) / P(B)
    let all_cards = list_all_cards();
    let hole_card_combs: Vec<Vec<&Card>> = all_cards.iter().combinations(2).collect();
    debug_assert_eq!(52 * 51 / 2, hole_card_combs.len());

    // P(h) / P(B) = 1 / (P(B | h0) + P(B | h1) + ... + P(B | h1325))
    let sum: f64 =
        hole_card_combs.iter().map(|hand| opponent_strategy.get_from_ref_slice(hand)).sum();
    assert!(sum > 0.0, "There is no hand with positive call/bet probability. In that case, there is no chance to play PostFlop game.");
    hole_card_combs
        .iter()
        .filter_map(|hand| {
            // P(h | B) = P(B|h) / (P(B | h0) + P(B | h1) + ... + P(B | h1325))
            let bet_call_prob = opponent_strategy.get_from_ref_slice(hand);
            if bet_call_prob == 0.0 {
                None
            } else {
                let phb: f64 = bet_call_prob / sum;
                Some(([*hand[0], *hand[1]], phb))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use more_asserts::assert_ge;

    use super::*;

    #[test]
    fn test_sb_strategy() {
        let cfg_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("configs/texas_holdem/headsup/preflop_sb_bet.txt");
        let strategy = PreflopStrategy::from_config(cfg_path);

        // AA >= 32o
        assert_ge!(
            strategy.get(&Card::from_str("Ah").unwrap(), &Card::from_str("As").unwrap()),
            strategy.get(&Card::from_str("3h").unwrap(), &Card::from_str("2s").unwrap())
        );

        // AA >= 32o
        assert_ge!(
            strategy.get(&Card::from_str("Ah").unwrap(), &Card::from_str("As").unwrap()),
            strategy.get(&Card::from_str("2h").unwrap(), &Card::from_str("3s").unwrap())
        );
    }

    #[test]
    fn test_bb_strategy() {
        let cfg_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("configs/texas_holdem/headsup/preflop_bb_call.txt");
        let strategy = PreflopStrategy::from_config(cfg_path);

        // AA >= 32o
        assert_ge!(
            strategy.get(&Card::from_str("Ah").unwrap(), &Card::from_str("As").unwrap()),
            strategy.get(&Card::from_str("3h").unwrap(), &Card::from_str("2s").unwrap())
        );

        // AA >= 32o
        assert_ge!(
            strategy.get(&Card::from_str("Ah").unwrap(), &Card::from_str("As").unwrap()),
            strategy.get(&Card::from_str("2h").unwrap(), &Card::from_str("3s").unwrap())
        );
    }

    #[test]
    fn test_postflop_reach_probabilities() {
        let mut strategy = PreflopStrategy::from_array([[0.0; card::RANK_COUNT]; card::RANK_COUNT]);
        // AA
        strategy.set(&Card::from_str("Ah").unwrap(), &Card::from_str("As").unwrap(), 1.0);
        // T8s
        strategy.set(&Card::from_str("Ts").unwrap(), &Card::from_str("8s").unwrap(), 0.2);

        let probs = preflop_strategy_to_post_flop_reach_probabilities(&strategy);
        // AAs (6 combinations) + T8s (4 combinations)
        assert_eq!(10, probs.len());
    }
}
