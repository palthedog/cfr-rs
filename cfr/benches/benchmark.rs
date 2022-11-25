use cfr::games::{
    dudo::DudoState,
    kuhn::KuhnState,
};
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

fn cfr_dudo_train_benchmark(c: &mut Criterion) {
    let mut trainer = cfr::Trainer::<DudoState>::new();
    c.bench_function("cfr::train<dudo> 10", |b| {
        b.iter(|| trainer.train(black_box(10)));
    });
}

fn cfr_kuhn_train_benchmark(c: &mut Criterion) {
    let mut trainer = cfr::Trainer::<KuhnState>::new();
    c.bench_function("cfr::train<kuhn> 10", |b| {
        b.iter(|| trainer.train(black_box(10)));
    });
}

criterion_group!(cfr_benches, cfr_dudo_train_benchmark, cfr_kuhn_train_benchmark);
criterion_main!(cfr_benches);
