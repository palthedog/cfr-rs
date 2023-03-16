use log::{
    debug,
    info,
    warn,
};

use super::*;

#[derive(Debug)]
pub struct Dealer {
    rule: Rule,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateResult {
    Keep,
    NextRound(Round),
    AllIn,
    NextHand,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HandResult {
    pub won_pots: Vec<i32>,
    pub hands: Vec<HandScore>,
}

impl HandResult {
    pub fn reflect_dealer_pos(&self, dealer_pos: usize) -> HandResult {
        let len = self.won_pots.len();
        let mut won_pots = Vec::with_capacity(len);
        let mut hands: Vec<HandScore> = Vec::with_capacity(len);
        won_pots.resize(len, 0);
        hands.resize(len, HandScore::empty());
        for i in 0..len {
            let actual = (i + dealer_pos) % len;
            won_pots[actual] = self.won_pots[i];
            hands[actual] = self.hands[i];
        }
        HandResult {
            won_pots,
            hands,
        }
    }
}

#[derive(Debug)]
pub struct GameResult {
    pub scores: Vec<i32>,
}

impl Dealer {
    pub fn new(rule: Rule) -> Dealer {
        Dealer {
            rule,
        }
    }

    pub fn update(&self, s: &mut HandState, act: Action) -> UpdateResult {
        let current = s.next_player;
        s.players[current as usize].took_action = true;
        s.last_action = Some(act);
        debug!("  Action: {:?}", act);
        match act {
            Action::Fold => {
                s.players[current as usize].folded = true;
            }
            Action::Call => s.players[current as usize].bet = s.max_bet(),
            Action::RaiseTo(raise_to) => {
                let mut raise_to = raise_to;
                if raise_to < s.round_state.min_raise_to
                    // it's not all-in
                    && raise_to < s.players[current as usize].stack
                {
                    warn!(
                        "    Invalid action min-raise: {}, received: {}",
                        s.round_state.min_raise_to, raise_to
                    );
                    raise_to = s.round_state.min_raise_to;
                };
                if raise_to >= s.players[current as usize].stack {
                    debug!("    Player {}: All-in", current);
                    raise_to = s.players[current as usize].stack;
                }

                let diff = raise_to - s.max_bet();
                s.round_state.min_raise_to = raise_to + diff;
                s.round_state.bet_cnt += 1;
                // Must update the bet after check diff.
                s.players[current as usize].bet = raise_to;
                debug!("    RoundState is updated: {:?}", s.round_state);
            }
        };

        // TODO: We must skip foled/all-in players.
        assert_eq!(2, s.players.len());
        s.next_player = (s.next_player + 1) % self.rule.player_cnt;

        let round_is_finished = s.round_is_finished();
        if s.hand_is_finished(round_is_finished) {
            UpdateResult::NextHand
        } else if s.everyone_all_in() {
            UpdateResult::AllIn
        } else if round_is_finished {
            let next_round = s.round.next();
            debug!("* Next Round: {:?} -> {:?}", s.round, next_round);
            UpdateResult::NextRound(next_round)
        } else {
            UpdateResult::Keep
        }
    }

    pub fn calculate_won_pots(&self, s: &HandState) -> HandResult {
        assert!(
            s.hand_is_finished(s.round_is_finished())
                || (s.everyone_all_in() && s.community_cards.len() == 5)
        );

        let mut scores = Vec::with_capacity(s.players.len());
        let mut max_score = HandScore::fold();
        for (i, player) in s.players.iter().enumerate() {
            let score;
            if !player.folded {
                score = hands::calc_player_score(s, player);
                info!("  score@{}: {}", i, score);
                max_score = max_score.max(score);
            } else {
                score = HandScore::fold();
                info!("  score@{}: fold", i);
            }
            scores.push(score);
        }

        let mut won_pots = Vec::with_capacity(s.players.len());
        let mut hands = Vec::with_capacity(s.players.len());
        let winner_cnt = scores.iter().filter(|&a| *a == max_score).count();
        let won_amount = s.pot() / winner_cnt as i32;
        info!("  pot: {}, won: {}, winner_cnt: {}", s.pot(), won_amount, winner_cnt);
        for (player, score) in s.players.iter().zip(scores.iter()) {
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

    pub fn init_round_and_deal_cards(&self, s: &mut HandState, deck: &mut Deck, round: Round) {
        self.init_round(s, round);
        self.deal_cards(s, deck);
    }

    pub fn init_round(&self, s: &mut HandState, round: Round) {
        for p in &mut s.players {
            // Note that all-in/folded players can't take new actions.
            if !(p.is_all_in() || p.is_folded()) {
                p.took_action = false;
            }
        }

        s.round = round;
        s.next_player = self.rule.first_player[s.round as usize];

        if s.round == Round::Preflop {
            s.last_action = None;
            s.players.resize(self.rule.player_cnt, PlayerState::default());
            for (i, blind) in self.rule.blinds.iter().enumerate() {
                s.players[i].bet = *blind;
            }
            let bb = self.rule.get_big_blind();
            s.round_state = RoundState {
                min_raise_to: bb * 2,
                bet_cnt: 0,
            };
            for player in &mut s.players {
                player.stack = self.rule.stack;
                player.folded = false;
            }
        } else {
            let next_player = &s.players[s.next_player];
            s.round_state = RoundState {
                min_raise_to: self.rule.get_big_blind() + next_player.bet,
                bet_cnt: 0,
            };
        }
    }

    pub fn handle_all_in(&self, s: &mut HandState, deck: &mut Deck) -> HandResult {
        let lack = 5 - s.community_cards.len();
        s.community_cards.append(&mut deck.draw_n(lack).to_vec());
        self.calculate_won_pots(s)
    }

    pub fn deal_cards(&self, s: &mut HandState, deck: &mut Deck) {
        match s.round {
            Round::Preflop => {
                for player in &mut s.players {
                    // TODO: No need of copy the vector?
                    player.hole_cards = deck.draw_n(2).to_vec();
                }
            }
            // TODO: ditto?
            Round::Flop => s.community_cards.append(&mut deck.draw_n(3).to_vec()),
            // TODO: ditto?
            Round::Turn | Round::River => s.community_cards.append(&mut deck.draw_n(1).to_vec()),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_round_state() {
        let dealer = Dealer::new(Rule::default());
        let mut hand_state = HandState::default();
        dealer.init_round(&mut hand_state, Round::Preflop);

        assert_eq!(100, hand_state.players[0].bet);
        assert_eq!(50, hand_state.players[1].bet);
        assert_eq!(150, hand_state.pot());
        assert_eq!(200, hand_state.round_state.min_raise_to);
        assert_eq!(0, hand_state.round_state.bet_cnt);

        // Player 1
        assert_eq!(1, hand_state.next_player);
        // Raise 1000
        let next = dealer.update(&mut hand_state, Action::RaiseTo(1000));
        assert_eq!(UpdateResult::Keep, next);
        assert_eq!(100, hand_state.players[0].bet);
        assert_eq!(1000, hand_state.players[1].bet);
        assert_eq!(1100, hand_state.pot());
        assert_eq!(1900, hand_state.round_state.min_raise_to);
        assert_eq!(1, hand_state.round_state.bet_cnt);

        // Player 0
        assert_eq!(0, hand_state.next_player);
        // Call
        let next = dealer.update(&mut hand_state, Action::Call);
        assert_eq!(UpdateResult::NextRound(Round::Flop), next);
        assert_eq!(1000, hand_state.players[0].bet);
        assert_eq!(1000, hand_state.players[1].bet);
        assert_eq!(2000, hand_state.pot());
        assert_eq!(1900, hand_state.round_state.min_raise_to);
        assert_eq!(1, hand_state.round_state.bet_cnt);

        // *** Flop ***
        dealer.init_round(&mut hand_state, Round::Flop);
        assert_eq!(Round::Flop, hand_state.round);
        assert_eq!(1000, hand_state.players[0].bet);
        assert_eq!(1000, hand_state.players[1].bet);
        assert_eq!(2000, hand_state.pot());
        assert_eq!(1100, hand_state.round_state.min_raise_to);
        assert_eq!(0, hand_state.round_state.bet_cnt);

        // Player 0
        assert_eq!(0, hand_state.next_player);
        // Raise to 1500 (raise by 500)
        let next = dealer.update(&mut hand_state, Action::RaiseTo(500 + 1000));
        assert_eq!(UpdateResult::Keep, next);
        assert_eq!(1500, hand_state.players[0].bet);
        assert_eq!(1000, hand_state.players[1].bet);
        assert_eq!(2500, hand_state.pot());
        assert_eq!(1500 + 500, hand_state.round_state.min_raise_to);
        assert_eq!(1, hand_state.round_state.bet_cnt);

        // Player 1
        assert_eq!(1, hand_state.next_player);
        // Raise to 2500 (Raise by 1000)
        let next = dealer.update(&mut hand_state, Action::RaiseTo(1500 + 1000));
        assert_eq!(UpdateResult::Keep, next);
        assert_eq!(1500, hand_state.players[0].bet);
        assert_eq!(2500, hand_state.players[1].bet);
        assert_eq!(4000, hand_state.pot());
        assert_eq!(2500 + 1000, hand_state.round_state.min_raise_to);
        assert_eq!(2, hand_state.round_state.bet_cnt);

        // Player 0
        assert_eq!(0, hand_state.next_player);
        // Call
        let next = dealer.update(&mut hand_state, Action::Call);
        assert_eq!(UpdateResult::NextRound(Round::Turn), next);
        assert_eq!(2500, hand_state.players[0].bet);
        assert_eq!(2500, hand_state.players[1].bet);
        assert_eq!(5000, hand_state.pot());
        assert_eq!(2, hand_state.round_state.bet_cnt);
    }

    #[test]
    fn test_all_in() {
        let dealer = Dealer::new(Rule::default());
        let mut hand_state = HandState::default();
        dealer.init_round(&mut hand_state, Round::Preflop);

        // Player 1
        assert_eq!(1, hand_state.next_player);
        // All in
        let next = dealer.update(&mut hand_state, Action::RaiseTo(20000));
        assert_eq!(20000, hand_state.players[1].bet);
        assert_eq!(UpdateResult::Keep, next);

        // Player 0
        assert_eq!(0, hand_state.next_player);
        // Call
        let next = dealer.update(&mut hand_state, Action::Call);
        assert_eq!(20000, hand_state.players[0].bet);
        assert_eq!(UpdateResult::AllIn, next);
    }

    #[test]
    fn test_update_too_much_raise() {
        let dealer = Dealer::new(Rule::default());
        let mut hand_state = HandState::default();
        dealer.init_round(&mut hand_state, Round::Preflop);

        assert_eq!(1, hand_state.next_player);
        dealer.update(&mut hand_state, Action::RaiseTo(19000));
        assert_eq!(19000 + 18900, hand_state.round_state.min_raise_to);

        assert_eq!(0, hand_state.next_player);
        // More than All-in.
        // It should be treated as All-in.
        dealer.update(&mut hand_state, Action::RaiseTo(40000));

        assert_eq!(19000, hand_state.players[1].bet);
        assert_eq!(20000, hand_state.players[0].bet);
    }

    #[test]
    fn test_update_too_low_raise() {
        let dealer = Dealer::new(Rule::default());
        let mut hand_state = HandState::default();
        dealer.init_round(&mut hand_state, Round::Preflop);

        assert_eq!(1, hand_state.next_player);
        // BB is 100
        // Raise to 1000
        // Raised BY 900
        dealer.update(&mut hand_state, Action::RaiseTo(1000));

        assert_eq!(1900, hand_state.round_state.min_raise_to);

        assert_eq!(0, hand_state.next_player);
        // More than All-in.
        // It should be treated as All-in.
        dealer.update(&mut hand_state, Action::RaiseTo(500));

        assert_eq!(1000, hand_state.players[1].bet);
        assert_eq!(1900, hand_state.players[0].bet);
    }

    #[test]
    fn test_play() {
        let dealer = Dealer::new(Rule::default());
        let mut hand_state = HandState::default();

        dealer.init_round(&mut hand_state, Round::Preflop);

        assert_eq!(1, hand_state.next_player);
        assert_eq!(UpdateResult::Keep, dealer.update(&mut hand_state, Action::Call));

        assert_eq!(0, hand_state.next_player);
        assert_eq!(
            UpdateResult::NextRound(Round::Flop),
            dealer.update(&mut hand_state, Action::Call)
        );
        dealer.init_round(&mut hand_state, Round::Flop);

        assert_eq!(0, hand_state.next_player);
        assert_eq!(UpdateResult::Keep, dealer.update(&mut hand_state, Action::Call));
    }
}
