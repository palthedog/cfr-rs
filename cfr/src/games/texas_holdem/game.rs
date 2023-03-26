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

pub struct TexasHoldemGame<S: RootNodeSampler> {
    dealer: Dealer,
    root_node_sampler: Option<S>,
}

pub type SubTreeId = usize;

/// An enum which represents a game tree node.
/// Note that the tree represents only a single hand (i.e. it cannot be used to represent a single table tournament)
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TexasHoldemNode {
    /// a.k.a. Root node for the entire game tree
    DealHands,

    /// Another root node used to solve a partial game tree.
    /// The very first chance action on the node leads the state to the root node of the partial game.
    /// For example, it is used as a root node of post-flop games.
    /// And this node has multiple child nodes with different pair of opponent's hole cards.
    SubTreeRoot,

    /// The dealer opens 3 community cards
    OpenFlop(Vec<TexasHoldemAction>, HandState),
    /// The dealer opens 1 community card
    OpenTurn(Vec<TexasHoldemAction>, HandState),
    /// The dealer open the last 1 community card
    OpenRiver(Vec<TexasHoldemAction>, HandState),
    /// Everyone did all-in. The dealer would open all community cards.
    EveryoneAllIn(Vec<TexasHoldemAction>, HandState),
    /// a.k.a. Terminal node
    TerminalNode(Vec<TexasHoldemAction>, HandState),

    /// A player takes an action
    PlayerNode(Vec<TexasHoldemAction>, HandState),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TexasHoldemAction {
    // Action for chance nodes
    DealHands(PlayerId, [Card; 2]),

    MoveToSubTreeRoot(SubTreeId),

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

pub trait RootNodeSampler {
    fn get_sub_tree_count(&self) -> usize;
    fn sample_sub_tree_id<R: Rng>(&self, rng: &mut R) -> SubTreeId;
    fn get_actions_to_sub_tree_root(&self, id: SubTreeId) -> Vec<TexasHoldemAction>;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TexasHoldemInfoSet {
    actions: Vec<TexasHoldemAction>,
}

impl fmt::Display for TexasHoldemInfoSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl<S: RootNodeSampler> Game for TexasHoldemGame<S> {
    type State = TexasHoldemNode;

    type InfoSet = TexasHoldemInfoSet;

    type Action = TexasHoldemAction;

    fn new_root(&self) -> Self::State {
        match &self.root_node_sampler {
            Some(_sampler) => TexasHoldemNode::SubTreeRoot,
            None => TexasHoldemNode::DealHands,
        }
    }

    fn to_info_set(&self, state: &Self::State) -> Self::InfoSet {
        // TODO: No need of copying the actions vector?
        //   it might be better to calculate a hash here and just store the hash value here?

        let action_history: &Vec<TexasHoldemAction> = match state {
            TexasHoldemNode::DealHands => panic!(),
            TexasHoldemNode::SubTreeRoot => panic!(),
            TexasHoldemNode::OpenFlop(acts, _) => acts,
            TexasHoldemNode::OpenTurn(acts, _) => acts,
            TexasHoldemNode::OpenRiver(acts, _) => acts,
            TexasHoldemNode::EveryoneAllIn(acts, _) => acts,
            TexasHoldemNode::TerminalNode(acts, _) => acts,
            TexasHoldemNode::PlayerNode(acts, _) => acts,
        };
        TexasHoldemInfoSet {
            actions: action_history.clone(),
        }
    }

    fn is_terminal(&self, state: &Self::State) -> bool {
        if let TexasHoldemNode::TerminalNode(_, _) = state {
            true
        } else {
            false
        }
    }

    fn get_payouts(&self, state: &Self::State) -> [f64; 2] {
        todo!()
    }

    fn get_node_player_id(&self, state: &Self::State) -> crate::games::PlayerId {
        if let TexasHoldemNode::PlayerNode(_, hand_state) = state {
            PlayerId::Player(hand_state.next_player)
        } else {
            PlayerId::Chance
        }
    }

    fn with_action(&self, state: &Self::State, action: Self::Action) -> Self::State {
        todo!()
    }

    fn list_legal_actions(&self, state: &Self::State) -> Vec<Self::Action> {
        todo!()
    }

    fn list_legal_chance_actions(&self, state: &Self::State) -> Vec<(Self::Action, f64)> {
        match state {
            TexasHoldemNode::DealHands => todo!(),
            TexasHoldemNode::SubTreeRoot => todo!(),
            TexasHoldemNode::OpenFlop(_, _) => todo!(),
            TexasHoldemNode::OpenTurn(_, _) => todo!(),
            TexasHoldemNode::OpenRiver(_, _) => todo!(),
            TexasHoldemNode::EveryoneAllIn(_, _) => todo!(),
            TexasHoldemNode::TerminalNode(_, _) => todo!(),
            TexasHoldemNode::PlayerNode(_, _) => todo!(),
        }
    }

    fn sample_chance_action<R: Rng>(&self, rng: &mut R, state: &Self::State) -> Self::Action {
        match state {
            TexasHoldemNode::SubTreeRoot => {
                let sub_tree_id = self.root_node_sampler.as_ref().unwrap().sample_sub_tree_id(rng);
                return TexasHoldemAction::MoveToSubTreeRoot(sub_tree_id);
            }

            TexasHoldemNode::DealHands => todo!(),
            TexasHoldemNode::OpenFlop(_, _) => todo!(),
            TexasHoldemNode::OpenTurn(_, _) => todo!(),
            TexasHoldemNode::OpenRiver(_, _) => todo!(),
            TexasHoldemNode::EveryoneAllIn(_, _) => todo!(),
            TexasHoldemNode::TerminalNode(_, _) => todo!(),
            TexasHoldemNode::PlayerNode(_, _) => todo!(),
        }

        /*
        let actions = self.list_legal_chance_actions(state);

        let dist =
            rand_distr::WeightedIndex::new(actions.iter().map(|p| p.1)).unwrap_or_else(|e| {
                panic!("Invalid weights: e: {} probs: {:?}", e, actions);
            });
        let index = dist.sample(rng);
        actions[index].0
        */
    }
}

impl<S: RootNodeSampler> TexasHoldemGame<S> {
    pub fn new(dealer: Dealer, root_node_sampler: Option<S>) -> Self {
        Self {
            dealer,
            root_node_sampler,
        }
    }

    /*
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
     */

    /*
        fn step(&mut self, state: &mut HandState, act: Action) -> Option<HandResult> {
            let next = self.dealer.update(&mut state, act);
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
