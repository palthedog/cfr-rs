use clap::{
    Parser,
    Subcommand,
    ValueEnum,
};

use cfr::{
    games,
    solvers::{self,},
};

#[derive(Parser)]
struct AppArgs {
    #[clap(long, short, value_enum)]
    game: Game,

    #[clap(subcommand)]
    solver: Solver,
}

#[derive(Subcommand)]
pub enum Solver {
    Cfr(solvers::cfr::TrainingArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Game {
    Kuhn,
    Dudo,
    Leduc,
}

fn main() {
    // Initialize env_logger with a default log level of INFO.
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = AppArgs::parse();
    match args.solver {
        Solver::Cfr(solver_args) => {
            match args.game {
                Game::Kuhn => {
                    let mut trainer = solvers::cfr::Trainer::<games::kuhn::KuhnState>::new();
                    trainer.train(&solver_args);
                }
                Game::Dudo => {
                    let mut trainer = solvers::cfr::Trainer::<games::dudo::DudoState>::new();
                    trainer.train(&solver_args);
                }
                Game::Leduc => {
                    let mut trainer = solvers::cfr::Trainer::<games::leduc::LeducState>::new();
                    trainer.train(&solver_args);
                }
            };
        }
    }
}
