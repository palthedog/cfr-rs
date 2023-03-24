use std::{
    collections::HashSet,
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
use rand_distr::WeightedAliasIndex;

use crate::games::Game;

use super::{
    card,
    Card,
    Dealer,
    HandState,
    TexasHoldemAction,
    TexasHoldemGame,
    TexasHoldemInfoSet,
    TexasHoldemNode,
};

pub struct PreflopStrategy {
    /// Preflop strategy is stored as `strategy[rank_y][rank_x]`.
    /// Just like as popular hand range tables
    /// bet/call probabilities for suited hands are stored in upper-right area (x > y) and
    /// ones for off-suited hands are stored in lower-left area (x <= y).
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

    pub fn get_from_slice(&self, cards: &[Card]) -> f64 {
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
    game: TexasHoldemGame,

    player_hand: [Card; 2],

    opponent_strategy: PreflopStrategy,
    opponent_hand_probabilities: Vec<([Card; 2], f64)>,
    opponent_hand_dist: WeightedAliasIndex<f64>,
}

impl TexasHoldemPostFlopGame {
    pub fn new(
        game: TexasHoldemGame,
        player_hand: [Card; 2],
        opponent_strategy: PreflopStrategy,
    ) -> Self {
        let consumed_cards: HashSet<Card> = player_hand.into();
        let opponent_hand_probabilities =
            preflop_strategy_to_post_flop_reach_probabilities(&consumed_cards, &opponent_strategy);

        let opponent_hand_dist = WeightedAliasIndex::new(
            opponent_hand_probabilities.iter().map(|(_hand, prob)| *prob).collect(),
        )
        .unwrap();
        Self {
            game,
            player_hand,
            opponent_strategy,
            opponent_hand_probabilities,
            opponent_hand_dist,
        }
    }
}

impl Game for TexasHoldemPostFlopGame {
    type State = <TexasHoldemGame as Game>::State;

    type InfoSet = <TexasHoldemGame as Game>::InfoSet;

    type Action = <TexasHoldemGame as Game>::Action;

    fn new_root(&self) -> Self::State {
        todo!()
    }

    fn to_info_set(&self, state: &Self::State) -> Self::InfoSet {
        self.game.to_info_set(state)
    }

    fn is_terminal(&self, state: &Self::State) -> bool {
        self.game.is_terminal(state)
    }

    fn get_payouts(&self, state: &Self::State) -> [f64; 2] {
        self.game.get_payouts(state)
    }

    fn get_node_player_id(&self, state: &Self::State) -> crate::games::PlayerId {
        self.game.get_node_player_id(state)
    }

    fn with_action(&self, state: &Self::State, action: Self::Action) -> Self::State {
        self.game.with_action(state, action)
    }

    fn list_legal_actions(&self, state: &Self::State) -> Vec<Self::Action> {
        self.game.list_legal_actions(state)
    }

    fn list_legal_chance_actions(&self, state: &Self::State) -> Vec<(Self::Action, f64)> {
        self.game.list_legal_chance_actions(state)
    }
}

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
    consumed_cards: &HashSet<Card>,
    opponent_strategy: &PreflopStrategy,
) -> Vec<([Card; 2], f64)> {
    // Compute P(h) / P(B)
    let all_possible_cards = list_all_cards().into_iter().filter(|c| !consumed_cards.contains(c));
    let hole_card_combs: Vec<Vec<Card>> = all_possible_cards.combinations(2).collect();
    debug_assert_eq!(52 * 51 / 2, hole_card_combs.len());

    // P(h) / P(B) = 1 / (P(B | h0) + P(B | h1) + ... + P(B | h1325))
    let sum: f64 = hole_card_combs.iter().map(|hand| opponent_strategy.get_from_slice(hand)).sum();
    assert!(sum > 0.0, "There is no hand with positive call/bet probability. In that case, there is no chance to play PostFlop game.");
    hole_card_combs
        .iter()
        .filter_map(|hand| {
            // P(h | B) = P(B|h) / (P(B | h0) + P(B | h1) + ... + P(B | h1325))
            let bet_call_prob = opponent_strategy.get_from_slice(hand);
            if bet_call_prob == 0.0 {
                None
            } else {
                let phb: f64 = bet_call_prob / sum;
                Some(([hand[0], hand[1]], phb))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use std::{
        collections::hash_set,
        str::FromStr,
    };

    use card::ch_rank;
    use more_asserts::{
        assert_ge,
        assert_lt,
    };

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
        const AA_ACT_PROB: f64 = 1.0;
        const T8S_ACT_PROB: f64 = 0.2;
        const OFF_SUITED_COMB: usize = 6; // sc, sh, sd, ch, cd, hd
        const SUITED_COMB: usize = 4; // s, c, h, d

        // AA
        strategy.set(&Card::from_str("Ah").unwrap(), &Card::from_str("As").unwrap(), AA_ACT_PROB);
        // T8s
        strategy.set(&Card::from_str("Ts").unwrap(), &Card::from_str("8s").unwrap(), T8S_ACT_PROB);

        let probs: Vec<([Card; 2], f64)> =
            preflop_strategy_to_post_flop_reach_probabilities(&Default::default(), &strategy);
        // AA (6 combinations) + T8s (4 combinations)
        assert_eq!(OFF_SUITED_COMB + SUITED_COMB, probs.len());

        let prob_sum: f64 = probs.iter().map(|(_hand, prob)| prob).sum();
        assert_lt!((1.0 - prob_sum).abs(), 1e-6);

        let aa_probs: f64 = probs
            .iter()
            .filter_map(|(hand, prob)| {
                if hand[0].rank == ch_rank('A') && hand[1].rank == ch_rank('A') {
                    Some(prob)
                } else {
                    None
                }
            })
            .sum();

        let aa_event_area = OFF_SUITED_COMB as f64 * AA_ACT_PROB;
        let t8s_event_area = SUITED_COMB as f64 * T8S_ACT_PROB;
        let expected_aa_probs = aa_event_area / (aa_event_area + t8s_event_area);
        assert_lt!(aa_probs - expected_aa_probs, 1.0e-6);
    }
}
