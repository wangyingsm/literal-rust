use criterion::{criterion_group, criterion_main, Criterion};
use num::integer::gcd;
use rand::{thread_rng, Rng};

fn bench_gcd(c: &mut Criterion) {
    c.bench_function("bench_gcd", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let numer = rng.gen_range(0..u64::MAX);
            let denom = rng.gen_range(1..u64::MAX);
            let _ = gcd(numer, denom);
        });
    });
}

criterion_group!(benches, bench_gcd);

criterion_main!(benches);
