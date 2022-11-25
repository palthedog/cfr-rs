use clap::Parser;

use cfr::games::dudo::DudoState;

#[derive(Parser)]
pub struct AppArgs {
    #[clap(long, short, value_parser, default_value_t = 1000)]
    iterations: u32,
}

fn main() {
    // Initialize env_logger with a default log level of INFO.
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = AppArgs::parse();

    let mut trainer = cfr::Trainer::<DudoState>::new();
    trainer.train(args.iterations);
}
