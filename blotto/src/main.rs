use blotto::Trainer;

fn main() {
    // Initialize env_logger with a default log level of INFO.
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let mut trainer = Trainer::new(5, 3);
    trainer.train(100_000);
}
