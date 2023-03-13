use std::{
    fs::File,
    io::{
        BufWriter,
        Write,
    },
    ops::Div,
    path::PathBuf,
    str::FromStr,
    time::Instant,
};

use clap::{
    Args,
    Parser,
    Subcommand,
    ValueEnum,
    ValueHint,
};

use cfr::{
    eval::compute_exploitability,
    games::{
        dudo::Dudo,
        kuhn::Kuhn,
        leduc::Leduc,
        Game,
    },
    solvers::{
        self,
        Solver,
    },
};
use humantime::Duration;
use log::info;

#[derive(Parser)]
struct AppArgs {
    #[clap(long, short, value_enum)]
    game: GameType,

    #[clap(flatten)]
    training_args: TrainingArgs,

    #[clap(subcommand)]
    solver: SolverArg,
}

#[derive(Args)]
struct TrainingArgs {
    #[clap(long, short, value_parser, default_value = "5s")]
    duration: Duration,

    #[clap(long, short, value_parser, value_hint(ValueHint::FilePath))]
    log_path: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SolverArg {
    Cfr(solvers::cfr::SolverArgs),
    MccfrExternalSampling(solvers::mccfr_external_sampling::SolverArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum GameType {
    Kuhn,
    Dudo,
    Leduc,
}

fn run<G, S>(game: G, training_args: TrainingArgs, solver_args: S::SolverArgs)
where
    G: Game,
    S: Solver<G>,
{
    let mut solver = S::new(game, solver_args);
    train(training_args, &mut solver);
}

fn train<G, S>(args: TrainingArgs, solver: &mut S)
where
    G: Game,
    S: Solver<G>,
{
    let mut log_writer = if let Some(path) = args.log_path {
        let f = File::create(path.clone()).unwrap_or_else(|err| {
            panic!("Failed to create a file: {:?}, {}", path, err);
        });
        let mut w = BufWriter::new(f);
        writeln!(w, "epoch,elapsed_seconds,touched_nodes,exploitability").expect("Failed to write");
        Some(w)
    } else {
        None
    };

    let log_file_freq = args.duration.div(100);
    let log_stdout_freq = Duration::from_str("10s").unwrap();
    let mut log_file_timer = Instant::now();
    let mut log_stdout_timer = Instant::now();
    let mut util = 0.0;
    let start_t = Instant::now();
    let mut i = 0u64;
    loop {
        util += solver.train_one_epoch();
        if start_t.elapsed() > *args.duration {
            break;
        }
        let log_file = log_writer.is_some() && log_file_timer.elapsed() > log_file_freq;
        let log_stdout = log_stdout_timer.elapsed() > *log_stdout_freq;
        if log_file || log_stdout {
            let exploitability = compute_exploitability(solver.game_ref(), solver);
            if log_stdout {
                info!(
                    "epoch {:10}: exploitability: {}",
                    i,
                    compute_exploitability(solver.game_ref(), solver)
                );
                info!("Average game value: {}", util / i as f64);
                log_stdout_timer = Instant::now();
            }

            if log_file {
                let w = log_writer.as_mut().unwrap();
                writeln!(
                    w,
                    "{},{},{},{:.12}",
                    i,
                    start_t.elapsed().as_secs(),
                    solver.get_touched_nodes_count(),
                    exploitability
                )
                .expect("Failed to write");
                w.flush().expect("Failed to flush");
                log_file_timer = Instant::now();
            }
        }
        i += 1;
    }
    info!("Training has finished");
    solver.print_strategy();

    // Save/log final result
    let exploitability = compute_exploitability(solver.game_ref(), solver);
    if let Some(mut w) = log_writer {
        writeln!(
            w,
            "{},{},{},{:.12}",
            i,
            start_t.elapsed().as_secs(),
            solver.get_touched_nodes_count(),
            exploitability
        )
        .expect("Failed to write");
        w.flush().expect("Failed to flush");
    }

    info!("Average game value: {}", util / i as f64);
    info!("exploitability: {}", exploitability);
}

macro_rules! def_solver {
    ($solver_t: ty, $game: expr, $($solver_args:expr),+) => {
        match $game {
            GameType::Kuhn => run::<Kuhn, $solver_t>(Kuhn::new(), $($solver_args),+),
            GameType::Dudo => run::<Dudo, $solver_t>(Dudo::new(), $($solver_args),+),
            GameType::Leduc => run::<Leduc, $solver_t>(Leduc::new(), $($solver_args),+),
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
            def_solver!(solvers::cfr::Trainer<_>, args.game, args.training_args, solver_args);
        }
        SolverArg::MccfrExternalSampling(solver_args) => {
            def_solver!(
                solvers::mccfr_external_sampling::Trainer::<_>,
                args.game,
                args.training_args,
                solver_args
            );
        }
    }
}
