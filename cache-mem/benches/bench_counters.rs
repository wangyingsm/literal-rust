use cache_mem::{emulate_counters, Counters, Matrix};
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_counters(c: &mut Criterion) {
    c.bench_function("bench_counters", |b| {
        let counters = Counters::default();
        b.iter(|| {
            emulate_counters(&counters);
        });
    });
}

fn bench_mul_matrix(c: &mut Criterion) {
    c.bench_function("bench_mul_matrix", |b| {
        let lhs = Matrix::from_random(1000);
        let rhs = Matrix::from_random(1000);
        b.iter(|| {
            let _ = lhs.mul_matrix(&rhs);
        })
    });
}

criterion_group!(benches, bench_counters, bench_mul_matrix);
criterion_main!(benches);
