use blotto::Trainer;
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

fn blotto_train_benchmark(c: &mut Criterion) {
    c.bench_function("blotto::train(5, 3) 10_000", |b| {
        let mut trainer = Trainer::new(5, 3);
        b.iter(|| trainer.train(black_box(10_000)))
    });
}

criterion_group!(blotto_benches, blotto_train_benchmark);
criterion_main!(blotto_benches);
