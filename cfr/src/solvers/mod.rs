use clap::Args;

use crate::{
    eval::Strategy,
    games::Game,
};

pub mod cfr;
pub mod mccfr_external_sampling;

pub trait Solver<G: Game>: Strategy<G> {
    type SolverArgs: Args;

    fn new(game: G, args: Self::SolverArgs) -> Self;
    fn game_ref(&self) -> &G;
    fn train_one_epoch(&mut self) -> f64;
    fn print_strategy(&self);
}
