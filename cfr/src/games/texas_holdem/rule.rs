// Note that it doesn't support limit holdem.
#[derive(Debug, Clone)]
pub struct Rule {
    // The stack size for each player at the start of each hand
    pub stack: i32,
    pub player_cnt: usize,
    pub blinds: Vec<i32>,
    // Who plays first?
    pub first_player: Vec<usize>,
}

impl Rule {
    pub fn new_2p_nolimit_reverse_blinds(stack: i32) -> Self {
        // Based on one in ACPC protocol: holdem.nolimit.2p.reverse_blinds.game
        Rule {
            stack,
            player_cnt: 2,

            blinds: vec![100, 50],
            first_player: vec![1, 0, 0, 0],
        }
    }

    pub fn get_big_blind(&self) -> i32 {
        let mut bb = 0;
        for b in &self.blinds {
            bb = bb.max(*b);
        }
        bb
    }
}
