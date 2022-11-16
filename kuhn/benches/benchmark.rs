use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

fn kuhn_train_benchmark(c: &mut Criterion) {
    let mut trainer = kuhn::Trainer::new();
    c.bench_function("kuhn::train 10_000", |b| {
        b.iter(|| trainer.train(black_box(10_000)));
    });
}

criterion_group!(kuhn_benches, kuhn_train_benchmark);
criterion_main!(kuhn_benches);
