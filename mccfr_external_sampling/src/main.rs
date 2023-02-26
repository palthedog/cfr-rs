use clap::{
    Parser,
    ValueEnum,
};

use cfr::games;
use mccfr_external_sampling::trainer::{
    Trainer,
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
            let mut trainer = Trainer::<games::kuhn::KuhnState>::new(args.training_args);
            trainer.train();
        }
        Game::Dudo => {
            let mut trainer = Trainer::<games::dudo::DudoState>::new(args.training_args);
            trainer.train();
        }
        Game::Leduc => {
            let mut trainer = Trainer::<games::leduc::LeducState>::new(args.training_args);
            trainer.train();
        }
    };
}
