fn main() {
    // Initialize env_logger with a default log level of INFO.
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let mut trainer = dudo::Trainer::new();
    trainer.train(100);
    //trainer.train(10_000);
    //trainer.train(10_000_000);  // it may take 5 hours
}
