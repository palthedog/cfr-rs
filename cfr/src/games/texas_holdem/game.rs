use std::fmt;

use crate::games::{
    Game,
    PlayerId,
};

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

pub struct TexasHoldemPostFlopGame {
    pub dealer: Dealer,
    pub hand_state: HandState,
    dealer_pos: usize,
    deck: Deck,
}

/// An enum which represents a game tree node.
/// Note that the tree represents only a single hand (i.e. it cannot be used to represent a single table tournament)
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TexasHoldemNode {
    /// a.k.a. Root node
    DealHands(HandState),
    /// A player takes an action
    PlayerNode(HandState),
    /// The dealer opens 3 community cards
    OpenFlop(HandState),
    /// The dealer opens 1 community card
    OpenTurn(HandState),
    /// The dealer open the last 1 community card
    OpenRiver(HandState),
    /// Everyone did all-in. The dealer would open all community cards.
    EveryoneAllIn(HandState),
    /// a.k.a. Terminal node
    TerminalNode(HandState),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TexasHoldemAction {
    // Action for chance nodes
    DealHands(PlayerId, [Card; 2]),

    OpenFlop([Card; 3]),
    OpenTurn(Card),
    OpenRiver(Card),

    HandleAllInAtPreFlop([Card; 5]),
    HandleAllInAtFlop([Card; 2]),
    HandleAllInAtTurn([Card; 1]),
    HandleAllInAtRiver(),

    // Action for player node
    PlayerAction(Action),
}

impl fmt::Display for TexasHoldemAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TexasHoldemInfoSet {}

impl fmt::Display for TexasHoldemInfoSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Game for TexasHoldemPostFlopGame {
    type State = TexasHoldemNode;

    type InfoSet = TexasHoldemInfoSet;

    type Action = TexasHoldemAction;

    fn new_root(&self) -> Self::State {
        todo!()
    }

    fn to_info_set(&self, state: &Self::State) -> Self::InfoSet {
        todo!()
    }

    fn is_terminal(&self, state: &Self::State) -> bool {
        todo!()
    }

    fn get_payouts(&self, state: &Self::State) -> [f64; 2] {
        todo!()
    }

    fn get_node_player_id(&self, state: &Self::State) -> crate::games::PlayerId {
        todo!()
    }

    fn with_action(&self, state: &Self::State, action: Self::Action) -> Self::State {
        todo!()
    }

    fn list_legal_actions(&self, state: &Self::State) -> Vec<Self::Action> {
        todo!()
    }

    fn list_legal_chance_actions(&self, _state: &Self::State) -> Vec<(Self::Action, f64)> {
        todo!();
    }
}

impl TexasHoldemPostFlopGame {
    /*
        fn new(dealer: Dealer, deck: Deck, players: Vec<Box<dyn Player>>) -> Self {
            Self {
                deck,
                dealer,
                dealer_pos: 0,
                hand_state: HandState::default(),
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
                NextHand => Some(self.hand_statecalculate_won_pots()),
            }
    }
    */
}

impl fmt::Display for TexasHoldemPostFlopGame {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.hand_state.dump())
    }
}
