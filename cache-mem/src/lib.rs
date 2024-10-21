use std::{
    ops::Index,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct Counters([Counter; 4]);

#[derive(Debug)]
#[repr(align(64))]
pub struct Counter(AtomicU64);

impl From<AtomicU64> for Counter {
    fn from(value: AtomicU64) -> Self {
        Self(value)
    }
}

impl Counters {
    pub fn new() -> Self {
        Self([
            AtomicU64::new(0).into(),
            AtomicU64::new(0).into(),
            AtomicU64::new(0).into(),
            AtomicU64::new(0).into(),
        ])
    }
}

impl Default for Counters {
    fn default() -> Self {
        Self::new()
    }
}

pub fn emulate_counters(counters: &Counters) {
    thread::scope(|s| {
        for i in 0..4_usize {
            s.spawn(move || {
                for _ in 0..1_000_000 {
                    counters.0[i].0.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
}

#[derive(Debug, PartialEq)]
pub struct Matrix {
    n: usize,
    data: Vec<f64>,
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0 * self.n + index.1]
    }
}

impl Matrix {
    pub fn from_random(n: usize) -> Self {
        let mut rng = thread_rng();
        let mut result = Vec::with_capacity(n * n);
        for _ in 0..n {
            for _ in 0..n {
                result.push(rng.gen());
            }
        }
        Self { n, data: result }
    }

    pub fn transpose(&self) -> Self {
        let mut data = Vec::with_capacity(self.n * self.n);
        for j in 0..self.n {
            for i in 0..self.n {
                data.push(self[(i, j)]);
            }
        }
        Self { n: self.n, data }
    }

    pub fn mul_matrix(&self, rhs: &Self) -> Self {
        assert_eq!(self.n, rhs.n);
        let rhs = rhs.transpose();
        let mut data = Vec::with_capacity(self.n * self.n);
        for i in 0..self.n {
            for j in 0..self.n {
                let mut result = 0.0;
                for k in 0..self.n {
                    result += self[(i, k)] * rhs[(j, k)];
                }
                data.push(result);
            }
        }
        Self { n: self.n, data }
    }
}

#[test]
fn test_mul_matrix() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let lhs = Matrix { n: 2, data };
    let data = vec![5.0, 6.0, 7.0, 8.0];
    let rhs = Matrix { n: 2, data };
    let data = vec![19.0, 22.0, 43.0, 50.0];
    let expected = Matrix { n: 2, data };
    assert_eq!(lhs.mul_matrix(&rhs), expected);
}
