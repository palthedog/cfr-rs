use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

fn dudo_train_benchmark(c: &mut Criterion) {
    let mut trainer = dudo::Trainer::new();
    c.bench_function("dudo::train 100", |b| {
        b.iter(|| trainer.train(black_box(100)));
    });
}

criterion_group!(dudo_benches, dudo_train_benchmark);
criterion_main!(dudo_benches);
