use std::fmt;

use crate::games::{Game, PlayerId};

use super::{cards::Cards, dealer::Dealer, *};

use itertools::Itertools;
use rand::Rng;

pub struct TexasHoldemGame<S: RootNodeSampler> {
    dealer: Dealer,
    root_node_sampler: Option<S>,
    abstraction: Abstraction,
}

pub type SubTreeId = usize;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeType {
    /// a.k.a. Root node for the entire game tree
    DealHands,

    /// Another root node used to solve a partial game tree.
    /// The very first chance action on the node leads the state to the root node of the partial game.
    /// For example, it is used as a root node of post-flop games.
    /// And this node has multiple child nodes with different pair of opponent's hole cards.
    SubTreeRoot,

    /// The dealer opens 3 community cards
    OpenFlop,
    /// The dealer opens 1 community card
    OpenTurn,
    /// The dealer open the last 1 community card
    OpenRiver,
    /// Everyone did all-in. The dealer would open all community cards.
    EveryoneAllIn,
    /// a.k.a. Terminal node
    TerminalNode,

    /// A player takes an action
    PlayerNode,
}

/// An enum which represents a game tree node.
/// Note that the tree represents only a single hand (i.e. it cannot be used to represent a single table tournament)
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TexasHoldemNode {
    pub node_type: NodeType,
    // Player action history which will be used to construct associated infoset.
    // We don't record chance actions because they could be hidden information from a player
    // (e.g. hole cards)
    pub player_action_history: Vec<TexasHoldemAction>,
    pub hand_state: HandState,
}

impl TexasHoldemNode {
    pub fn new_root() -> Self {
        TexasHoldemNode {
            node_type: NodeType::DealHands,
            player_action_history: vec![],
            hand_state: HandState::default(),
        }
    }

    pub fn new_sub_tree_root() -> Self {
        TexasHoldemNode {
            node_type: NodeType::SubTreeRoot,
            player_action_history: vec![],
            hand_state: HandState::default(),
        }
    }
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
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

pub trait RootNodeSampler {
    fn get_sub_tree_count(&self) -> usize;
    fn get_sub_tree_reach_probabilities(&self) -> &[f64];
    fn sample_sub_tree_id<R: Rng>(&self, rng: &mut R) -> SubTreeId;
    fn get_hand_state_at_sub_tree_root(&self, id: SubTreeId) -> HandState;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TexasHoldemInfoSet {
    hole_cards: Vec<Card>,
    community_cards: Vec<Card>,
    player_actions: Vec<TexasHoldemAction>,
}

impl fmt::Display for TexasHoldemInfoSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<S: RootNodeSampler> Game for TexasHoldemGame<S> {
    type State = TexasHoldemNode;

    type InfoSet = TexasHoldemInfoSet;

    type Action = TexasHoldemAction;

    fn new_root(&self) -> Self::State {
        match &self.root_node_sampler {
            Some(_sampler) => TexasHoldemNode::new_sub_tree_root(),
            None => TexasHoldemNode::new_root(),
        }
    }

    fn to_info_set(&self, state: &Self::State) -> Self::InfoSet {
        // TODO: No need of copying the actions vector?
        //   it might be better to calculate a hash here and just store the hash value here?
        TexasHoldemInfoSet {
            hole_cards: state.hand_state.players[state.hand_state.next_player].hole_cards.clone(),
            community_cards: state.hand_state.community_cards.clone(),
            player_actions: state.player_action_history.clone(),
        }
    }

    fn is_terminal(&self, state: &Self::State) -> bool {
        state.node_type == NodeType::TerminalNode
    }

    fn get_payouts(&self, state: &Self::State) -> [f64; 2] {
        let result = self.dealer.calculate_won_pots(&state.hand_state);
        assert_eq!(2, result.won_pots.len());
        // Normalize payouts by big blind
        let big_blind: f64 = *self.dealer.get_rule().blinds.iter().max().unwrap() as f64;
        [result.won_pots[0] as f64 / big_blind, result.won_pots[1] as f64 / big_blind]
    }

    fn get_node_player_id(&self, state: &Self::State) -> crate::games::PlayerId {
        if state.node_type == NodeType::PlayerNode {
            PlayerId::Player(state.hand_state.next_player)
        } else {
            PlayerId::Chance
        }
    }

    fn with_action(&self, state: &Self::State, action: Self::Action) -> Self::State {
        // Action history which is visible from BOTH players.
        let mut new_history = state.player_action_history.clone();

        let (node_type, next_hand_state) = match action {
            TexasHoldemAction::DealHands(_, _) => todo!(),
            TexasHoldemAction::MoveToSubTreeRoot(sub_tree_id) => {
                let root_state = self
                    .root_node_sampler
                    .as_ref()
                    .unwrap()
                    .get_hand_state_at_sub_tree_root(sub_tree_id);
                (NodeType::PlayerNode, root_state)
            }
            TexasHoldemAction::OpenFlop(cards) => {
                let mut next_hand_state = state.hand_state.clone();
                next_hand_state.community_cards.extend_from_slice(&cards);
                self.dealer.init_round(&mut next_hand_state, Round::Flop);
                assert_eq!(
                    3,
                    next_hand_state.community_cards.len(),
                    "state: {:?}, action: {:?}",
                    state,
                    action
                );
                (NodeType::PlayerNode, next_hand_state)
            }
            TexasHoldemAction::OpenTurn(card) => {
                let mut next_hand_state = state.hand_state.clone();
                next_hand_state.community_cards.push(card);
                self.dealer.init_round(&mut next_hand_state, Round::Turn);
                assert_eq!(
                    4,
                    next_hand_state.community_cards.len(),
                    "state: {:?}, action: {:?}",
                    state,
                    action
                );
                (NodeType::PlayerNode, next_hand_state)
            }
            TexasHoldemAction::OpenRiver(card) => {
                let mut next_hand_state = state.hand_state.clone();
                next_hand_state.community_cards.push(card);
                self.dealer.init_round(&mut next_hand_state, Round::River);
                assert_eq!(
                    5,
                    next_hand_state.community_cards.len(),
                    "state: {:?}, action: {:?}",
                    state,
                    action
                );
                (NodeType::PlayerNode, next_hand_state)
            }
            TexasHoldemAction::HandleAllInAtPreFlop(_) => todo!(),
            TexasHoldemAction::HandleAllInAtFlop(cards) => {
                let mut next_hand_state = state.hand_state.clone();
                next_hand_state.community_cards.extend_from_slice(&cards);
                assert_eq!(
                    5,
                    next_hand_state.community_cards.len(),
                    "state: {:?}, action: {:?}",
                    state,
                    action
                );
                (NodeType::TerminalNode, next_hand_state)
            }
            TexasHoldemAction::HandleAllInAtTurn(cards) => {
                let mut next_hand_state = state.hand_state.clone();
                next_hand_state.community_cards.extend_from_slice(&cards);
                assert_eq!(
                    5,
                    next_hand_state.community_cards.len(),
                    "state: {:?}, action: {:?}",
                    state,
                    action
                );
                (NodeType::TerminalNode, next_hand_state)
            }
            TexasHoldemAction::HandleAllInAtRiver() => {
                panic!();
            }
            TexasHoldemAction::PlayerAction(act) => {
                assert_eq!(state.node_type, NodeType::PlayerNode);

                new_history.push(action);

                let mut next_state = state.hand_state.clone();
                let update_result = self.dealer.update(&mut next_state, act);
                let node_type = match update_result {
                    UpdateResult::Keep => NodeType::PlayerNode,
                    UpdateResult::NextRound(next_round) => match next_round {
                        Round::Preflop => todo!(),
                        Round::Flop => NodeType::OpenFlop,
                        Round::Turn => NodeType::OpenTurn,
                        Round::River => NodeType::OpenRiver,
                    },
                    UpdateResult::AllIn => NodeType::EveryoneAllIn,
                    // Caller must call .init later.
                    UpdateResult::NextHand => NodeType::TerminalNode,
                };
                (node_type, next_state)
            }
        };
        TexasHoldemNode {
            node_type,
            player_action_history: new_history,
            hand_state: next_hand_state,
        }
    }

    fn list_legal_actions(&self, state: &Self::State) -> Vec<Self::Action> {
        match state.node_type {
            NodeType::PlayerNode => self.abstraction.list_actions(&state.hand_state),
            NodeType::DealHands
            | NodeType::SubTreeRoot
            | NodeType::OpenFlop
            | NodeType::OpenTurn
            | NodeType::OpenRiver
            | NodeType::EveryoneAllIn
            | NodeType::TerminalNode => {
                panic!("list_legal_actions is called on a chance node: {:?}", state);
            }
        }
    }

    fn list_legal_chance_actions(&self, state: &Self::State) -> Vec<(Self::Action, f64)> {
        let mut acts = vec![];
        match state.node_type {
            NodeType::DealHands => todo!(),
            NodeType::SubTreeRoot => {
                let probs =
                    self.root_node_sampler.as_ref().unwrap().get_sub_tree_reach_probabilities();
                for (id, prob) in probs.iter().enumerate() {
                    acts.push((TexasHoldemAction::MoveToSubTreeRoot(id), *prob));
                }
            }
            NodeType::OpenFlop => {
                let available_cards = state.hand_state.get_available_cards().to_vec();
                let comb = available_cards.into_iter().combinations(3).collect_vec();
                let unif_prob = 1.0 / comb.len() as f64;
                for opened in &comb {
                    let act = TexasHoldemAction::OpenFlop([opened[0], opened[1], opened[2]]);
                    acts.push((act, unif_prob));
                }
            }
            NodeType::OpenTurn => {
                let available_cards = state.hand_state.get_available_cards().to_vec();
                let unif_prob = 1.0 / available_cards.len() as f64;
                for card in available_cards.into_iter() {
                    let act = TexasHoldemAction::OpenTurn(card);
                    acts.push((act, unif_prob));
                }
            }
            NodeType::OpenRiver => {
                let available_cards = state.hand_state.get_available_cards().to_vec();
                let unif_prob = 1.0 / available_cards.len() as f64;
                for card in available_cards.into_iter() {
                    let act = TexasHoldemAction::OpenRiver(card);
                    acts.push((act, unif_prob));
                }
            }
            NodeType::EveryoneAllIn => {
                let comm_len = state.hand_state.community_cards.len();
                let deal_cnt = 5 - comm_len;
                if deal_cnt == 0 {
                    // Players are on river
                    panic!("EveryoneAllIn node shouldn't be used if everyone all-in in river. Use terminal node instead.");
                }

                let available_cards = state.hand_state.get_available_cards().to_vec();
                let comb = available_cards.into_iter().combinations(deal_cnt).collect_vec();
                let unif_prob = 1.0 / comb.len() as f64;
                for opened in &comb {
                    let act = match deal_cnt {
                        5 => TexasHoldemAction::HandleAllInAtPreFlop([
                            opened[0], opened[1], opened[2], opened[3], opened[4],
                        ]),
                        2 => TexasHoldemAction::HandleAllInAtFlop([opened[0], opened[1]]),
                        1 => TexasHoldemAction::HandleAllInAtTurn([opened[0]]),
                        _ => panic!("Unknown deal_cnt: {} when we list all chance actions for EveryoneAllIn", deal_cnt),
                    };
                    acts.push((act, unif_prob));
                }
            }
            NodeType::TerminalNode => todo!(),
            NodeType::PlayerNode => {
                panic!("list_legal_chance_actions is called on a player node: {:?}", state);
            }
        }
        acts
    }

    fn sample_chance_action<R: Rng>(&self, rng: &mut R, state: &Self::State) -> Self::Action {
        match state.node_type {
            NodeType::DealHands => todo!(),
            NodeType::SubTreeRoot => {
                let sub_tree_id = self.root_node_sampler.as_ref().unwrap().sample_sub_tree_id(rng);
                TexasHoldemAction::MoveToSubTreeRoot(sub_tree_id)
            }
            NodeType::OpenFlop => {
                let mut available_cards: Cards = state.hand_state.get_available_cards();
                TexasHoldemAction::OpenFlop([
                    available_cards.sample_card(rng),
                    available_cards.sample_card(rng),
                    available_cards.sample_card(rng),
                ])
            }
            NodeType::OpenTurn => {
                let mut available_cards: Cards = state.hand_state.get_available_cards();
                TexasHoldemAction::OpenTurn(available_cards.sample_card(rng))
            }
            NodeType::OpenRiver => {
                let mut available_cards: Cards = state.hand_state.get_available_cards();
                TexasHoldemAction::OpenRiver(available_cards.sample_card(rng))
            }
            NodeType::EveryoneAllIn => {
                let mut available_cards: Cards = state.hand_state.get_available_cards();

                match state.hand_state.community_cards.len() {
                    0 => TexasHoldemAction::HandleAllInAtPreFlop([
                        available_cards.sample_card(rng),
                        available_cards.sample_card(rng),
                        available_cards.sample_card(rng),
                        available_cards.sample_card(rng),
                        available_cards.sample_card(rng),
                    ]),
                    3 => TexasHoldemAction::HandleAllInAtFlop([
                        available_cards.sample_card(rng),
                        available_cards.sample_card(rng),
                    ]),
                    4 => TexasHoldemAction::HandleAllInAtTurn([available_cards.sample_card(rng)]),
                    5 => TexasHoldemAction::HandleAllInAtRiver(),
                    _ => panic!(
                        "Invalid number of community cards: {}, {:?}",
                        state.hand_state.community_cards.len(),
                        state.hand_state
                    ),
                }
            }
            NodeType::TerminalNode => todo!(),
            NodeType::PlayerNode => {
                panic!("sample_chance_action is called on a player node: {:?}", state);
            }
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
    pub fn new(dealer: Dealer, abstraction: Abstraction, root_node_sampler: Option<S>) -> Self {
        Self {
            dealer,
            abstraction,
            root_node_sampler,
        }
    }

    /*
        fn apply_player_action(
            &self,
            history: &Vec<TexasHoldemAction>,
            hand_state: &HandState,
            action: Action,
        ) -> TexasHoldemNode {
            let mut next_state = hand_state.clone();
            let next = self.dealer.update(&mut next_state, action);
            match next {
                Keep => TexasHoldemNode::PlayerNode(next_state),
                NextRound(next_round) => {
                    self.dealer.init_round_and_deal_cards(next_state, &mut self.deck, next_round);
                    None
                }
                AllIn => Some(self.dealer.handle_all_in(&mut self.hand_state, &mut self.deck)),
                // Caller must call .init later.
                NextHand => Some(self.hand_statecalculate_won_pots()),
            }
    }
        */

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

        fn step(&mut self, state: &mut HandState, act: Action) -> Option<HandResult> {
            let next = self.dealer.update(&mut state, act);
            match next {
                Keep => None,
                NextRound(next_round) => {
                    self.dealer.init_round_and_deal_cards(state, &mut self.deck, next_round);
                    None
                }
                AllIn => Some(self.dealer.handle_all_in(&mut self.hand_state, &mut self.deck)),
                // Caller must call .init later.
                NextHand => Some(self.hand_statecalculate_won_pots()),
            }
    }
             */
}
