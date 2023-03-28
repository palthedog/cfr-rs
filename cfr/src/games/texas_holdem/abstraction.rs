use std::collections::HashSet;

use more_asserts::{
    assert_gt,
};

use super::{
    Action,
    HandState,
    TexasHoldemAction,
};

pub enum BetSize {
    Minimum,
    RelativeToPot(f64),
}

pub struct Abstraction {
    bet_sizes: Vec<BetSize>,
    raise_sizes: Vec<BetSize>,
}

impl Abstraction {
    pub fn new(bet_sizes: Vec<BetSize>, raise_sizes: Vec<BetSize>) -> Self {
        assert_gt!(bet_sizes.len(), 0);
        assert_gt!(raise_sizes.len(), 0);
        Self {
            bet_sizes,
            raise_sizes,
        }
    }

    pub fn new_basic() -> Self {
        Self::new(
            vec![BetSize::Minimum, BetSize::RelativeToPot(0.5), BetSize::RelativeToPot(1.0)],
            vec![BetSize::Minimum, BetSize::RelativeToPot(0.5), BetSize::RelativeToPot(1.0)],
        )
    }

    pub fn list_actions(&self, state: &HandState) -> Vec<TexasHoldemAction> {
        let call_amount_by = state.max_bet() - state.players[state.next_player].bet;
        // Pot size "if" the player called
        let pot_size = state.pot() + call_amount_by;
        let rules = if state.round_state.bet_cnt == 0 {
            &self.bet_sizes
        } else {
            &self.raise_sizes
        };

        let mut set = HashSet::new();
        let mut actions = vec![];
        for rule in rules {
            let amount = match rule {
                BetSize::Minimum => state.round_state.min_raise_to,
                BetSize::RelativeToPot(r) => {
                    let addition = (pot_size as f64 * r).floor() as i32;
                    state.max_bet() + addition
                }
            };
            if amount < state.round_state.min_raise_to {
                continue;
            }
            if set.insert(amount) {
                actions.push(TexasHoldemAction::PlayerAction(Action::RaiseTo(amount)));
            }
        }
        actions
    }
}

#[cfg(test)]
mod tests {

    use super::super::{
        PlayerState,
        Round,
        RoundState,
    };

    use super::*;

    fn new_hand_state(bets: [i32; 2], min_raise_to: i32) -> HandState {
        let base_player_state = PlayerState {
            stack: 1000000,
            bet: 0,
            took_action: false,
            folded: false,
            hole_cards: vec![], // it doesn't matter
        };
        let bet_cnt = if bets[0] == bets[1] {
            0
        } else {
            1
        };

        HandState {
            next_player: 0,
            last_action: None,
            round: Round::Flop,
            round_state: RoundState {
                min_raise_to,
                bet_cnt,
            },
            community_cards: vec![],
            players: vec![
                PlayerState {
                    bet: bets[0],
                    ..base_player_state.clone()
                },
                PlayerState {
                    bet: bets[1],
                    ..base_player_state
                },
            ],
        }
    }

    #[test]
    fn test_double_pot_size_bet() {
        let min_raise_to = 200;
        let current_bet = 100;

        let state = new_hand_state([current_bet, current_bet], min_raise_to);
        let abstraction = Abstraction::new(
            // bet
            vec![BetSize::Minimum, BetSize::RelativeToPot(2.0)],
            // raise
            vec![BetSize::Minimum],
        );
        let bet_sizes = abstraction.list_actions(&state);

        let pot_size = 200;
        assert_eq!(pot_size, state.pot());
        assert_eq!(
            vec![
                // 200
                TexasHoldemAction::PlayerAction(Action::RaiseTo(min_raise_to)),
                // 300
                TexasHoldemAction::PlayerAction(Action::RaiseTo(current_bet + pot_size * 2)),
            ],
            bet_sizes
        );
    }

    #[test]
    fn test_pot_size_bet_duplicated() {
        let min_raise_to = 200;
        let current_bet = 100;

        let state = new_hand_state([current_bet, current_bet], min_raise_to);
        let abstraction = Abstraction::new(
            // bet
            vec![
                BetSize::Minimum,            // 200 (min_raise_to)
                BetSize::Minimum,            // simply duplicated enum
                BetSize::RelativeToPot(0.5), // 200 = 100 (current_bet) + 100 (pot_size / 2)
                BetSize::RelativeToPot(1.0), // 300
            ],
            // raise
            vec![BetSize::Minimum],
        );
        let bet_sizes = abstraction.list_actions(&state);

        let pot_size = 200;
        assert_eq!(pot_size, state.pot());
        assert_eq!(
            vec![
                // 200
                TexasHoldemAction::PlayerAction(Action::RaiseTo(min_raise_to)),
                // 200: duplicated action shouldn't be included
                // TexasHoldemAction::PlayerAction(Action::RaiseTo(min_raise_to)),
                // 200: duplicated action shouldn't be included
                // TexasHoldemAction::PlayerAction(Action::RaiseTo(current_bet + pot_size / 2)),
                // 300
                TexasHoldemAction::PlayerAction(Action::RaiseTo(current_bet + pot_size)),
            ],
            bet_sizes
        );
    }
}
