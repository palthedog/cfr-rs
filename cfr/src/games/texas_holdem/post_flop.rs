use std::{
    fs::File,
    io::{
        BufRead,
        BufReader,
    },
    path::PathBuf,
};

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
        assert!(sum > 0.0, "There is no hand with positive call/bet probability. In that case, there is no chance to play PostFlop game.");

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
        assert!(sum > 0.0, "There is no hand with positive call/bet probability. In that case, there is no chance to play PostFlop game.");
        Self {
            strategy,
        }
    }

    pub fn get(&self, cards: [Card; 2]) -> f64 {
        self.get_from_ranks(cards[0].rank, cards[1].rank, cards[0].suit == cards[1].suit)
    }

    pub fn get_from_ranks(&self, rank0: card::Rank, rank1: card::Rank, suited: bool) -> f64 {
        let stronger = rank0.max(rank1);
        let weaker = rank0.min(rank1);
        if suited {
            let x = card::rank_to_index(stronger);
            let y = card::rank_to_index(weaker);
            self.strategy[y][x]
        } else {
            let y = card::rank_to_index(stronger);
            let x = card::rank_to_index(weaker);
            self.strategy[y][x]
        }
    }
}

pub struct TexasHoldemPostFlopGame {
    pub dealer: Dealer,
    pub hand_state: HandState,
}

impl TexasHoldemPostFlopGame {}

pub fn preflop_strategy_to_post_flop_reach_probabilities(opponent_strategy: &PreflopStrategy) {}

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
            strategy.get([Card::from_str("Ah").unwrap(), Card::from_str("As").unwrap()]),
            strategy.get([Card::from_str("3h").unwrap(), Card::from_str("2s").unwrap()])
        );

        // AA >= 32o
        assert_ge!(
            strategy.get([Card::from_str("Ah").unwrap(), Card::from_str("As").unwrap()]),
            strategy.get([Card::from_str("2h").unwrap(), Card::from_str("3s").unwrap()])
        );
    }

    #[test]
    fn test_bb_strategy() {
        let cfg_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("configs/texas_holdem/headsup/preflop_bb_call.txt");
        let strategy = PreflopStrategy::from_config(cfg_path);

        // AA >= 32o
        assert_ge!(
            strategy.get([Card::from_str("Ah").unwrap(), Card::from_str("As").unwrap()]),
            strategy.get([Card::from_str("3h").unwrap(), Card::from_str("2s").unwrap()])
        );

        // AA >= 32o
        assert_ge!(
            strategy.get([Card::from_str("Ah").unwrap(), Card::from_str("As").unwrap()]),
            strategy.get([Card::from_str("2h").unwrap(), Card::from_str("3s").unwrap()])
        );
    }
}
