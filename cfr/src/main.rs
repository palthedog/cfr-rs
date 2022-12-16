use clap::{
    Parser,
    ValueEnum,
};

use cfr::{
    games,
    TrainingArgs,
};

#[derive(Parser)]
struct AppArgs {
    #[clap(long, short, value_enum)]
    game: Game,

    #[clap(flatten)]
    training_args: TrainingArgs,
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

    match args.game {
        Game::Kuhn => {
            let mut trainer = cfr::Trainer::<games::kuhn::KuhnState>::new();
            trainer.train(&args.training_args);
        }
        Game::Dudo => {
            let mut trainer = cfr::Trainer::<games::dudo::DudoState>::new();
            trainer.train(&args.training_args);
        }
        Game::Leduc => {
            let mut trainer = cfr::Trainer::<games::leduc::LeducState>::new();
            trainer.train(&args.training_args);
        }
    };
}
