use clap::Args;

use crate::games::State;

pub mod cfr;
pub mod mccfr_external_sampling;

pub trait Solver<G: State> {
    type SolverArgs: Args;

    fn new(args: Self::SolverArgs) -> Self;
    fn train(&mut self);
}
