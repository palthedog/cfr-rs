use std::fmt::Display;

use more_asserts::{
    assert_ge,
    assert_gt,
    assert_le,
    assert_lt,
    debug_assert_gt,
};
use rand::Rng;

use super::{
    GameState,
    PlayerId,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DudoAction {
    Claim(Claim),
    Dudo,

    ChanceRollDices([RollResult; 2]),
}

impl Display for DudoAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DudoAction::Claim(c) => write!(f, "{}x{}", c.count, c.rank + 1),
            DudoAction::Dudo => write!(f, "Dudo"),
            DudoAction::ChanceRollDices(_) => write!(f, "RollDices"),
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
    pub fn new_none() -> Self {
        Self {
            count: [-1; 6],
        }
    }

    pub fn new(count: [i32; 6]) -> Self {
        Self {
            count,
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DudoState {
    pub round: u32,
    pub node_player_id: PlayerId,
    pub prev_winner: PlayerId,
    pub action_history: Vec<DudoAction>,
    pub player_rolls: [RollResult; 2],
    pub dice_count: [i32; 2],
}

impl GameState for DudoState {
    type InfoSet = DudoInfoSet;
    type Action = DudoAction;

    fn new_root() -> Self {
        Self {
            round: 0,
            node_player_id: PlayerId::Chance,
            prev_winner: PlayerId::Player(0),
            action_history: vec![],
            player_rolls: [RollResult::new_none(), RollResult::new_none()],
            dice_count: [1, 1],
        }
    }

    fn to_info_set(&self) -> DudoInfoSet {
        DudoInfoSet::from(self)
    }

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

    fn get_node_player_id(&self) -> PlayerId {
        self.node_player_id
    }

    fn with_action(&self, action: Self::Action) -> Self {
        let mut next = self.clone();
        next.update(action);
        next
    }

    fn list_legal_chance_actions(&self) -> Vec<(Self::Action, f64)> {
        let mut v = vec![];
        let num_actions = 6 * 6;
        let prob = 1.0 / num_actions as f64;
        for p in 0..6 {
            let mut ps = [0; 6];
            ps[p] = 1;
            let p_result = RollResult::new(ps);
            for o in 0..6 {
                let mut os = [0; 6];
                os[o] = 1;
                let o_result = RollResult::new(os);
                let act = DudoAction::ChanceRollDices([p_result, o_result]);
                v.push((act, prob));
            }
        }
        assert_eq!(num_actions, v.len());
        v
    }

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

impl DudoState {
    fn update(&mut self, action: DudoAction) {
        match action {
            DudoAction::Claim(c) => self.update_claim(&c),
            DudoAction::Dudo => self.update_dudo(),
            DudoAction::ChanceRollDices(roll_result) => self.update_chance(roll_result),
        }
    }

    fn opponent_player_id(&self, _player_id: PlayerId) -> PlayerId {
        match self.node_player_id {
            PlayerId::Chance => panic!(),
            PlayerId::Player(i) => PlayerId::Player(i ^ 1),
        }
    }

    fn update_chance(&mut self, roll_result: [RollResult; 2]) {
        self.player_rolls = roll_result;
        self.node_player_id = self.prev_winner;
    }

    fn update_claim(&mut self, claim: &Claim) {
        if !self.action_history.is_empty() {
            debug_assert_gt!(*claim, self.current_claim().unwrap());
        }
        self.node_player_id = self.opponent_player_id(self.node_player_id);
        self.action_history.push(DudoAction::Claim(*claim));
    }

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

    fn update_dudo(&mut self) {
        let challenger = self.node_player_id;
        let challenged = self.opponent_player_id(self.node_player_id);

        let challenged_claim = self.current_claim().unwrap();

        let actual_dice_count: i32 =
            self.player_rolls.iter().map(|roll| roll.count_dice(challenged_claim.rank)).sum();
        let claimed_dice_count = challenged_claim.count;
        let loser: PlayerId;
        match actual_dice_count.cmp(&claimed_dice_count) {
            std::cmp::Ordering::Equal => {
                // challenger loses
                loser = challenger;
                self.dice_count[loser.index()] -= 1;
            }
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
        }
        self.prev_winner = self.opponent_player_id(loser);
        self.action_history.clear();
        self.round += 1;
    }
}

#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct DudoInfoSet {
    pub uid: u64,
    pub round: u32,
    pub next_player_id: PlayerId,
    pub action_history: Vec<DudoAction>,
    pub player_roll: RollResult,
    pub dice_count: [i32; 2],
}

impl From<&DudoState> for DudoInfoSet {
    fn from(state: &DudoState) -> Self {
        assert_ne!(state.node_player_id, PlayerId::Chance);
        let mut uid: u64 = 0;
        // max: 12 loops * 5 = 60 bits
        assert_le!(state.action_history.len(), 12);
        for i in 0..12 {
            let bits: u64 = match state.action_history.get(i) {
                None => 0,
                Some(DudoAction::Claim(c)) => {
                    // count: [0, 2] -> 2 bits
                    // rank: [0, 5] -> 3 bits
                    // | count (2) | rank (3) |
                    assert_gt!(c.count, 0);
                    assert_le!(c.count, 2);
                    assert_ge!(c.rank, 0);
                    assert_lt!(c.rank, 6);
                    ((c.count as u64) << 3) | c.rank as u64
                }
                Some(_) => todo!(),
            };
            assert_le!(bits, 0b11111);
            uid = (uid << 5) | bits;
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
        // round: 1 bit
        assert_le!(state.round, 1);
        uid = (uid << 1) | state.round as u64;

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

impl PartialEq for DudoInfoSet {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Display for DudoInfoSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{} ", self.round)?;
        write!(f, "{:?}", self.next_player_id)?;
        for (i, cnt) in self.player_roll.count.iter().enumerate() {
            for _ in 0..*cnt {
                write!(f, " {}", i + 1)?;
            }
        }
        write!(f, ", acts[")?;
        for act in self.action_history.iter() {
            write!(f, "{}, ", act)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payoffs() {
        let mut state = DudoState::new_root();
        let chance = DudoAction::ChanceRollDices([
            RollResult::new([1, 0, 0, 0, 0, 0]),
            RollResult::new([0, 1, 0, 0, 0, 0]),
        ]);
        state.update(chance);

        let claim1x1 = DudoAction::Claim(Claim {
            count: 1,
            rank: 0,
        });
        state.update(claim1x1);

        let dudo = DudoAction::Dudo;
        state.update(dudo);
        assert_eq!([1.0, -1.0], state.get_payouts());
    }
}
