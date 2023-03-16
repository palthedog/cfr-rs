#[derive(Debug, Clone)]
pub struct Rule {
    // The stack size for each player at the start of each hand
    pub stack: i32,
    pub player_cnt: usize,
    pub blinds: Vec<i32>,
    pub first_player: Vec<usize>,
}

impl Default for Rule {
    fn default() -> Rule {
        Rule {
            stack: 20_000,
            player_cnt: 2,

            blinds: vec![100, 50],
            first_player: vec![1, 0, 0, 0],
        }
    }
}

impl Rule {
    pub fn get_big_blind(&self) -> i32 {
        let mut bb = 0;
        for b in &self.blinds {
            bb = bb.max(*b);
        }
        bb
    }
}
