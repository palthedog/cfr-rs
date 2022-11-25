use std::fmt::Display;

use more_asserts::{
    assert_gt,
    debug_assert_gt,
};
use rand::Rng;

use super::{
    InfoSet,
    PlayerId,
    State,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DudoAction {
    Claim(Claim),
    Dudo,
}

impl Display for DudoAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DudoAction::Claim(c) => write!(f, "{}x{}", c.count, c.rank + 1),
            DudoAction::Dudo => write!(f, "Dudo"),
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
    pub fn new_rand(rng: &mut impl Rng, dice_count: i32) -> Self {
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

#[derive(Debug, Clone)]
pub struct DudoState {
    pub round: u32,
    pub node_player_id: PlayerId,
    pub prev_winner: PlayerId,
    pub action_history: Vec<DudoAction>,
    pub player_rolls: [RollResult; 2],
    pub dice_count: [i32; 2],
}

impl State for DudoState {
    type InfoSet = DudoInfoSet;

    fn is_terminal(&self) -> bool {
        self.dice_count.iter().any(|cnt| *cnt == 0)
    }

    fn get_payouts(&self) -> [f64; 2] {
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

    fn to_info_set(&self) -> DudoInfoSet {
        DudoInfoSet::from(self)
    }

    fn get_node_player_id(&self) -> PlayerId {
        self.node_player_id
    }

    fn new_root<R: Rng>(rng: &mut R) -> Self {
        let dice_count = 1;
        DudoState::new_root(
            [RollResult::new_rand(rng, dice_count), RollResult::new_rand(rng, dice_count)],
            dice_count,
        )
    }

    fn with_action(&self, action: <DudoInfoSet as InfoSet>::Action) -> Self {
        let mut next = self.clone();
        next.update(action);
        next
    }
}

impl DudoState {
    pub fn new_root(player_rolls: [RollResult; 2], dice_count: i32) -> Self {
        Self {
            round: 0,
            node_player_id: PlayerId::Player(0),
            prev_winner: PlayerId::Player(0),
            action_history: vec![],
            player_rolls,
            dice_count: [dice_count, dice_count],
        }
    }

    fn update(&mut self, action: DudoAction) {
        match action {
            DudoAction::Claim(c) => self.update_claim(&c),
            DudoAction::Dudo => self.update_dudo(),
        }
    }

    fn opponent_player_id(&self, _player_id: PlayerId) -> PlayerId {
        match self.node_player_id {
            PlayerId::Chance => panic!(),
            PlayerId::Player(i) => PlayerId::Player(i ^ 1),
        }
    }

    fn update_claim(&mut self, claim: &Claim) {
        if !self.action_history.is_empty() {
            debug_assert_gt!(*claim, self.current_claim());
        }
        self.node_player_id = self.opponent_player_id(self.node_player_id);
        self.action_history.push(DudoAction::Claim(*claim));
    }

    fn update_dudo(&mut self) {
        let challenger = self.node_player_id;
        let challenged = self.opponent_player_id(self.node_player_id);

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
                self.dice_count[loser.index()] = 0.max(self.dice_count[loser.index()] - diff);
            }
            std::cmp::Ordering::Less => {
                // the actual count is less than the challenged claim
                // challenger wins
                loser = challenged;
                let diff = claimed_dice_count - actual_dice_count;
                assert_gt!(diff, 0);
                self.dice_count[loser.index()] = 0.max(self.dice_count[loser.index()] - diff);
            }
            std::cmp::Ordering::Equal => {
                // challenger loses
                loser = challenger;
                self.dice_count[loser.index()] -= 1;
            }
        }
        self.prev_winner = self.opponent_player_id(loser);
        self.action_history.clear();
        self.round += 1;
    }

    fn current_claim(&self) -> Claim {
        if let DudoAction::Claim(claim) = self.action_history.last().unwrap() {
            *claim
        } else {
            panic!()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DudoInfoSet {
    pub uid: u64,
    pub round: u32,
    pub next_player_id: PlayerId,
    pub action_history: Vec<DudoAction>,
    pub player_roll: RollResult,
    pub dice_count: [i32; 2],
}

impl InfoSet for DudoInfoSet {
    type Action = DudoAction;

    fn list_legal_actions(&self) -> Vec<DudoAction> {
        let mut v = vec![];

        if !self.action_history.is_empty() {
            // dudo
            v.push(DudoAction::Dudo);
        }

        let count_max: i32 = self.dice_count.iter().sum();

        let rank_start: usize;
        let normalized_count: i32;
        let last_claim = self.current_claim();
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
                v.push(DudoAction::Claim(Claim {
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
                v.push(DudoAction::Claim(c));
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
                v.push(DudoAction::Claim(c));
            }
        }
        v
    }
}

impl DudoInfoSet {
    fn current_claim(&self) -> Option<Claim> {
        if self.action_history.is_empty() {
            return None;
        }
        if let DudoAction::Claim(claim) = self.action_history.last().unwrap() {
            Some(*claim)
        } else {
            None
        }
    }
}

impl From<&DudoState> for DudoInfoSet {
    fn from(state: &DudoState) -> Self {
        assert_ne!(state.node_player_id, PlayerId::Chance);
        let mut uid: u64 = 0;
        // max: 12 loops * 5 = 60 bits
        for act in state.action_history.iter() {
            match act {
                DudoAction::Claim(c) => {
                    uid = (uid << 2) | c.count as u64; // count: [0, 2] -> 2 bits
                    uid = (uid << 3) | c.rank as u64; // rank: [0, 5] -> 3 bits
                }
                _ => todo!(),
            }
        }
        // dice: [0, 5] 3 bits
        for (dice, cnt) in
            state.player_rolls[state.get_node_player_id().index()].count.iter().enumerate()
        {
            if *cnt == 1 {
                uid = (uid << 3) | dice as u64;
                break;
            }
        }

        Self {
            uid,
            round: state.round,
            next_player_id: state.node_player_id,
            action_history: state.action_history.clone(),
            player_roll: state.player_rolls[state.get_node_player_id().index()],
            dice_count: state.dice_count,
        }
    }
}

impl std::hash::Hash for DudoInfoSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

impl Display for DudoInfoSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Info set
        write!(f, "{:?}", self.next_player_id)?;
        write!(f, "{:?}  ", self.player_roll.count)?;
        write!(f, "[")?;
        for act in self.action_history.iter() {
            write!(f, "{}, ", act)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
