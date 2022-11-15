use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

fn rps_train_benchmark(c: &mut Criterion) {
    c.bench_function("rps::train 10_000", |b| {
        b.iter(|| rps::train(black_box(10_000)))
    });
}

criterion_group!(rps_benches, rps_train_benchmark);
criterion_main!(rps_benches);
