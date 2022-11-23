use std::{
    collections::HashMap,
    fmt::Display,
};

use log::{
    debug,
    info,
};
use more_asserts::{
    assert_gt,
    debug_assert_ge,
    debug_assert_gt,
};
use rand::{
    rngs::ThreadRng,
    thread_rng,
    Rng,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Claim(Claim),
    Dudo,

    DiceRoll([RollResult; 2]),
}

impl Action {}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Claim(c) => write!(f, "{}x{}", c.count, c.rank + 1),
            Action::Dudo => write!(f, "Dudo"),
            Action::DiceRoll(_) => write!(f, "DiceRoll"),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Claim {
    pub count: i32,
    pub rank: usize,
}

impl Claim {
    pub fn normalized_count(&self) -> i32 {
        if self.rank == 0 {
            self.count * 2
        } else {
            self.count
        }
    }
}

impl PartialOrd for Claim {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Claim {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let cmp_n = self.normalized_count().cmp(&other.normalized_count());
        if cmp_n.is_ne() {
            return cmp_n;
        }
        self.rank.cmp(&other.rank)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RollResult {
    count: [i32; 6],
}

impl RollResult {
    pub fn new_rand(rng: &mut impl Rng, dice_count: usize) -> Self {
        let mut result = RollResult {
            count: [0; 6],
        };
        for _ in 0..dice_count {
            let dice = rng.gen_range(0..6);
            result.count[dice] += 1;
        }
        result
    }

    pub fn count_dice(&self, dice: usize) -> i32 {
        if dice == 0 {
            self.count[0]
        } else {
            self.count[0] + self.count[dice]
        }
    }
}

impl Display for RollResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for dice in 0..6 {
            for _ in 0..self.count[dice] {
                write!(f, "{}", dice + 1)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerId {
    Player0,
    Player1,
    Chance,
}

impl PlayerId {
    pub fn opponent(&self) -> PlayerId {
        match self {
            PlayerId::Player0 => PlayerId::Player1,
            PlayerId::Player1 => PlayerId::Player0,
            PlayerId::Chance => panic!(),
        }
    }
}

impl From<PlayerId> for usize {
    fn from(id: PlayerId) -> Self {
        match id {
            PlayerId::Player0 => 0,
            PlayerId::Player1 => 1,
            PlayerId::Chance => panic!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InfoSet {
    pub uid: u64,
    pub round: u32,
    pub next_player_id: PlayerId,
    pub actions: Vec<Action>,
    pub player_roll: RollResult,
    pub dice_count: [i32; 2],
}

impl InfoSet {
    fn current_claim(&self) -> Option<Claim> {
        if self.actions.is_empty() {
            return None;
        }
        if let Action::Claim(claim) = self.actions.last().unwrap() {
            Some(*claim)
        } else {
            None
        }
    }
}

impl From<&State> for InfoSet {
    fn from(state: &State) -> Self {
        assert_ne!(state.next_player_id, PlayerId::Chance);
        let mut uid: u64 = 0;
        // max: 12 loops * 5 = 60 bits
        for act in state.actions.iter() {
            match act {
                Action::Claim(c) => {
                    uid = (uid << 2) | c.count as u64; // count: [0, 2] -> 2 bits
                    uid = (uid << 3) | c.rank as u64; // rank: [0, 5] -> 3 bits
                }
                _ => todo!(),
            }
        }
        // dice: [0, 5] 3 bits
        for (dice, cnt) in
            state.player_rolls[state.next_player_id as usize].count.iter().enumerate()
        {
            if *cnt == 1 {
                uid = (uid << 3) | dice as u64;
                break;
            }
        }

        Self {
            uid,
            round: state.round,
            next_player_id: state.next_player_id,
            actions: state.actions.clone(),
            player_roll: state.player_rolls[state.next_player_id as usize],
            dice_count: state.dice_count,
        }
    }
}

impl std::hash::Hash for InfoSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub round: u32,
    pub next_player_id: PlayerId,
    pub prev_winner: PlayerId,
    pub actions: Vec<Action>,
    pub player_rolls: [RollResult; 2],
    pub dice_count: [i32; 2],
}

impl State {
    pub fn new_root(player_rolls: [RollResult; 2], dice_count: i32) -> Self {
        Self {
            round: 0,
            next_player_id: PlayerId::Player0,
            prev_winner: PlayerId::Player0,
            actions: vec![],
            player_rolls,
            dice_count: [dice_count, dice_count],
        }
    }

    pub fn with_action(&self, action: Action) -> Self {
        let mut next = self.clone();
        next.update(action);
        next
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Claim(c) => self.update_claim(&c),
            Action::Dudo => self.update_dudo(),
            Action::DiceRoll(rolls) => self.update_dice_roll(rolls),
        }
    }

    fn update_dice_roll(&mut self, rolls: [RollResult; 2]) {
        self.next_player_id = self.prev_winner;
        self.player_rolls = rolls;
    }

    fn update_claim(&mut self, claim: &Claim) {
        if !self.actions.is_empty() {
            debug_assert_gt!(*claim, self.current_claim());
        }
        self.next_player_id = self.next_player_id.opponent();
        self.actions.push(Action::Claim(*claim));
    }

    fn update_dudo(&mut self) {
        let challenger = self.next_player_id;
        let challenged = self.next_player_id.opponent();

        let challenged_claim = self.current_claim();

        let actual_dice_count: i32 =
            self.player_rolls.iter().map(|roll| roll.count_dice(challenged_claim.rank)).sum();
        let claimed_dice_count = challenged_claim.normalized_count();
        let loser: PlayerId;
        match actual_dice_count.cmp(&claimed_dice_count) {
            std::cmp::Ordering::Greater => {
                // the actual count exceeds the challenged claim
                // challenger loses
                loser = challenger;
                let diff = actual_dice_count - claimed_dice_count;
                assert_gt!(diff, 0);
                self.dice_count[loser as usize] = 0.max(self.dice_count[loser as usize] - diff);
            }
            std::cmp::Ordering::Less => {
                // the actual count is less than the challenged claim
                // challenger wins
                loser = challenged;
                let diff = claimed_dice_count - actual_dice_count;
                assert_gt!(diff, 0);
                self.dice_count[loser as usize] = 0.max(self.dice_count[loser as usize] - diff);
            }
            std::cmp::Ordering::Equal => {
                // challenger loses
                loser = challenger;
                self.dice_count[loser as usize] -= 1;
            }
        }
        self.prev_winner = loser.opponent();
        self.actions.clear();
        self.round += 1;
    }

    fn current_claim(&self) -> Claim {
        if let Action::Claim(claim) = self.actions.last().unwrap() {
            *claim
        } else {
            panic!()
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.dice_count.iter().any(|cnt| *cnt == 0)
    }

    pub fn get_payouts(&self) -> [f64; 2] {
        debug_assert!(self.is_terminal());

        let mut ret = [0.0; 2];
        for (i, cnt) in self.dice_count.iter().enumerate() {
            ret[i] = if *cnt == 0 {
                -1.0
            } else {
                1.0
            };
        }
        ret
    }
}

#[derive(Clone)]
pub struct Node {
    regret_sum: Vec<f64>,
    strategy: Vec<f64>,
    strategy_sum: Vec<f64>,

    info_set: InfoSet,
}

impl Node {
    pub fn new(info_set: InfoSet) -> Self {
        Self {
            regret_sum: vec![],
            strategy: vec![],
            strategy_sum: vec![],
            info_set,
        }
    }

    pub fn list_legal_actions(&self) -> Vec<Action> {
        let mut v = vec![];

        if !self.info_set.actions.is_empty() {
            // dudo
            v.push(Action::Dudo);
        }

        let count_max: i32 = self.info_set.dice_count.iter().sum();

        let rank_start: usize;
        let normalized_count: i32;
        let last_claim = self.info_set.current_claim();
        match last_claim {
            Some(c) => {
                rank_start = c.rank + 1;
                normalized_count = c.normalized_count();
            }
            None => {
                rank_start = 0;
                normalized_count = 0;
            }
        }

        // same count higher rank
        if normalized_count > 0 && normalized_count <= count_max {
            for rank in rank_start..6 {
                v.push(Action::Claim(Claim {
                    count: normalized_count,
                    rank,
                }));
            }
        }
        // higher count
        {
            // rank == one
            let count_start: i32 = normalized_count / 2 + 1;
            for count in count_start..count_max + 1 {
                let c = Claim {
                    count,
                    rank: 0,
                };
                if last_claim.is_some() {
                    assert_gt!(c, last_claim.unwrap());
                }
                v.push(Action::Claim(c));
            }
        }
        // excludes one
        for rank in 1..6 {
            for count in normalized_count + 1..count_max + 1 {
                let c = Claim {
                    count,
                    rank,
                };
                if last_claim.is_some() {
                    assert_gt!(c, last_claim.unwrap());
                }
                v.push(Action::Claim(c));
            }
        }
        v
    }

    pub fn to_strategy(&mut self, realization_weight: f64) -> Vec<f64> {
        let normalizing_sum: f64 = self.regret_sum.iter().filter(|v| **v >= 0.0).sum();
        let actions_len = self.strategy.len();
        if normalizing_sum == 0.0 {
            self.strategy = vec![1.0 / actions_len as f64; actions_len];
        } else {
            for (i, reg) in self.regret_sum.iter().enumerate() {
                self.strategy[i] = if *reg >= 0.0 {
                    *reg / normalizing_sum
                } else {
                    0.0
                };
            }
        };

        for i in 0..actions_len {
            debug_assert_ge!(self.strategy[i], 0.0);
            self.strategy_sum[i] += realization_weight * self.strategy[i];
        }

        // How can I prevent cloneing the array here?
        self.strategy.clone()
    }

    pub fn to_average_strategy(&self) -> Vec<f64> {
        let normalizing_sum: f64 = self.strategy_sum.iter().sum();
        if normalizing_sum == 0.0 {
            let actions_len = self.strategy.len();
            return vec![1.0 / actions_len as f64; actions_len];
        }
        self.strategy_sum.iter().map(|s| s / normalizing_sum).collect()
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Info set
        write!(f, "{:?}", self.info_set.next_player_id)?;
        write!(f, "{:?}  ", self.info_set.player_roll.count)?;
        write!(f, "[")?;
        for act in self.info_set.actions.iter() {
            write!(f, "{}, ", act)?;
        }
        write!(f, "]")?;

        // avg Strategy
        let avg_strategy = self.to_average_strategy();
        let actions = self.list_legal_actions();
        write!(f, " Avg Strategy[")?;
        for i in 0..actions.len() {
            write!(f, "{}: {:.03}, ", actions[i], avg_strategy[i])?;
        }
        write!(f, "]")?;

        // regrets
        /*
        write!(f, " Regret Sum [")?;
        for i in 0..actions.len() {
            write!(f, "{}: {:.08}, ", actions[i], self.regret_sum[i])?;
        }
        write!(f, "]")?;
        */

        Ok(())
    }
}

pub struct Trainer {
    rng: ThreadRng,
    nodes: HashMap<InfoSet, Node>,
}

impl Trainer {
    pub fn new() -> Self {
        Trainer {
            rng: thread_rng(),
            nodes: HashMap::new(),
        }
    }

    pub fn cfr(&mut self, state: &State, actions_prob: [f64; 2]) -> [f64; 2] {
        if state.is_terminal() {
            return state.get_payouts();
        }

        let player = state.next_player_id;
        if player == PlayerId::Chance {
            let chance_action = Action::DiceRoll([
                RollResult::new_rand(&mut self.rng, state.dice_count[0] as usize),
                RollResult::new_rand(&mut self.rng, state.dice_count[1] as usize),
            ]);
            let next_state = state.with_action(chance_action);
            return self.cfr(&next_state, actions_prob);
        }

        let info_set = InfoSet::from(state);
        let node =
            self.nodes.entry(info_set.clone()).or_insert_with(|| Node::new(info_set.clone()));
        let mut node_util = [0.0f64; 2];

        let actions = node.list_legal_actions();
        let actions_len = actions.len();
        assert_gt!(actions_len, 0);
        debug!("CFR state: {:#?}", state);
        debug!("legal actions: {:#?}", actions);

        if node.strategy.is_empty() {
            // initialize buffers
            node.strategy.resize(actions_len, 0.0);
            node.strategy_sum.resize(actions_len, 0.0);
            node.regret_sum.resize(actions_len, 0.0);
        }

        let mut action_utils = vec![0.0; actions_len]; // Note: allocating array on the stack is faster.
        let realization_weight = actions_prob[player as usize];
        let strategy = node.to_strategy(realization_weight);
        for (i, act) in actions.iter().enumerate() {
            let action_prob = strategy[i];
            let next_state = state.with_action(*act);
            let mut next_actions_prob = actions_prob;

            next_actions_prob[player as usize] *= action_prob;

            let action_util = self.cfr(&next_state, next_actions_prob);
            action_utils[i] = action_util[player as usize];
            for (player, player_action_util) in action_util.iter().enumerate() {
                node_util[player] += action_prob * player_action_util;
            }
        }

        let opponent = player.opponent();
        let node = self.nodes.get_mut(&info_set).unwrap();
        for i in 0..actions_len {
            let regret: f64 = action_utils[i] - node_util[player as usize];
            let opponent_prob = actions_prob[opponent as usize];
            node.regret_sum[i] += opponent_prob * regret;
        }

        node_util
    }

    pub fn train(&mut self, iterations: u32) {
        let mut rng = rand::thread_rng();
        let mut util = 0.0;
        const DICE_COUNT: usize = 1;
        for i in 0..iterations {
            if i != 0 && i % 10000 == 0 {
                info!("epoch {}: Average game value: {}", i, util / i as f64);
            }
            let initial = State::new_root(
                [
                    RollResult::new_rand(&mut rng, DICE_COUNT),
                    RollResult::new_rand(&mut rng, DICE_COUNT),
                ],
                DICE_COUNT as i32,
            );
            util += self.cfr(&initial, [1.0, 1.0])[PlayerId::Player0 as usize];
        }
        info!("Training has finished");

        let mut nodes: Vec<Node> = self.nodes.values().cloned().collect();
        nodes.sort_by_key(|n| {
            (n.info_set.actions.len(), n.info_set.actions.clone(), n.info_set.player_roll)
        });
        info!("Nodes [");
        for node in nodes {
            if node.list_legal_actions().len() > 1 {
                info!("    {}", node);
            }
        }
        info!("]");

        info!("Average game value: {}", util / iterations as f64);
    }
}
