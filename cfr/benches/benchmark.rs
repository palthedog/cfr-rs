use cfr::{
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
use criterion::{
    criterion_group,
    criterion_main,
    Criterion,
};

fn new_cfr<G: Game>(game: G) -> solvers::cfr::Trainer<G> {
    solvers::cfr::Trainer::<G>::new(game, solvers::cfr::SolverArgs {})
}

fn new_mccfr_external_ampling<G: Game>(game: G) -> solvers::mccfr_external_sampling::Trainer<G> {
    solvers::mccfr_external_sampling::Trainer::<G>::new(game, Default::default())
}

fn cfr_train_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("cfr group");

    group.bench_function("cfr::train<dudo>", |b| {
        let mut trainer = new_cfr(Dudo::new());
        b.iter(|| trainer.train_one_epoch());
    });

    group.bench_function("cfr::train<kuhn>", |b| {
        let mut trainer = new_cfr(Kuhn::new());
        b.iter(|| trainer.train_one_epoch());
    });

    group.bench_function("cfr::train<leduc>", |b| {
        let mut trainer = new_cfr(Leduc::new());
        b.iter(|| trainer.train_one_epoch());
    });

    group.finish();
}

fn mccfr_external_sampling_train_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("mccfr-external-sampling group");

    group.bench_function("mccfr-external-sampling::train<dudo>", |b| {
        let mut trainer = new_mccfr_external_ampling(Dudo::new());
        b.iter(|| trainer.train_one_epoch());
    });

    group.bench_function("mccfr-external-sampling::train<kuhn>", |b| {
        let mut trainer = new_mccfr_external_ampling(Kuhn::new());
        b.iter(|| trainer.train_one_epoch());
    });

    group.bench_function("mccfr-external-sampling::train<leduc>", |b| {
        let mut trainer = new_mccfr_external_ampling(Leduc::new());
        b.iter(|| trainer.train_one_epoch());
    });

    group.finish();
}

criterion_group!(cfr_benches, cfr_train_benchmark, mccfr_external_sampling_train_benchmark);
criterion_main!(cfr_benches);
