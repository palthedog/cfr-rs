use clap::{
    Parser,
    Subcommand,
    ValueEnum,
};

use cfr::{
    games::{
        dudo::DudoState,
        kuhn::KuhnState,
        leduc::LeducState,
        State,
    },
    solvers::{
        self,
        Solver,
    },
};

#[derive(Parser)]
struct AppArgs {
    #[clap(long, short, value_enum)]
    game: Game,

    #[clap(subcommand)]
    solver: SolverArg,
}

#[derive(Subcommand)]
pub enum SolverArg {
    Cfr(solvers::cfr::TrainingArgs),
    MccfrExternalSampling(solvers::mccfr_external_sampling::TrainingArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Game {
    Kuhn,
    Dudo,
    Leduc,
}

fn run<G, S>(args: S::SolverArgs)
where
    G: State,
    S: Solver<G>,
{
    let mut trainer = S::new(args);
    trainer.train();
}

macro_rules! def_solver {
    ($solver_t: ty, $game: expr, $($solver_args:expr),+) => {
        match $game {
            Game::Kuhn => run::<KuhnState, $solver_t>($($solver_args),+),
            Game::Dudo => run::<DudoState, $solver_t>($($solver_args),+),
            Game::Leduc => run::<LeducState, $solver_t>($($solver_args),+),
        };
    };
}

fn main() {
    // Initialize env_logger with a default log level of INFO.
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = AppArgs::parse();
    match args.solver {
        SolverArg::Cfr(solver_args) => {
            def_solver!(solvers::cfr::Trainer<_>, args.game, solver_args);
        }
        SolverArg::MccfrExternalSampling(solver_args) => {
            def_solver!(solvers::mccfr_external_sampling::Trainer::<_>, args.game, solver_args);
        }
    }
}
