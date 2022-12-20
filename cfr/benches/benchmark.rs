use cfr::{
    games::{
        dudo::DudoState,
        kuhn::KuhnState,
        leduc::LeducState,
    },
    TrainingArgs,
};
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

fn cfr_train_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Trainer::train group");

    group.bench_function("cfr::train<dudo> 10", |b| {
        let mut trainer = cfr::Trainer::<DudoState>::new();
        b.iter(|| trainer.train(black_box(&TrainingArgs::new(10))));
    });

    group.bench_function("cfr::train<kuhn> 10", |b| {
        let mut trainer = cfr::Trainer::<KuhnState>::new();
        b.iter(|| trainer.train(black_box(&TrainingArgs::new(10))));
    });

    group.bench_function("cfr::train<leduc> 10", |b| {
        let mut trainer = cfr::Trainer::<LeducState>::new();
        b.iter(|| trainer.train(black_box(&TrainingArgs::new(10))));
    });

    group.finish();
}

criterion_group!(cfr_benches, cfr_train_benchmark);
criterion_main!(cfr_benches);
