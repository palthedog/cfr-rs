use clap::Args;

use crate::{
    eval::Strategy,
    games::GameState,
};

pub mod cfr;
pub mod mccfr_external_sampling;

pub trait Solver<G: GameState>: Strategy<G> {
    type SolverArgs: Args;

    fn new(args: Self::SolverArgs) -> Self;
    fn train_one_epoch(&mut self) -> f64;
    fn print_strategy(&self);
}
