fn main() {
    // Initialize env_logger with a default log level of INFO.
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let mut trainer = dudo::Trainer::new();
    //trainer.train(100);
    trainer.train(100_000); // it takes 2 mins

    //trainer.train(500_000); // it takes 10 mins
    //trainer.train(10_000_000); // it may take 2.5 hours
}
