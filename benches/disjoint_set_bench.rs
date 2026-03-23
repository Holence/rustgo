use criterion::{Criterion, criterion_group, criterion_main};
use rustgo::DisjointSet;
use rustgo::IdxTrait;
use std::hint::black_box;

#[derive(Clone, Copy)]
enum Op {
    Insert(usize),
    Connect(usize, usize),
    DeleteGroup(usize),
}

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn generate_ops(n: usize, seed: u64) -> Vec<Op> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut ops = Vec::with_capacity(n);

    for _ in 0..n {
        let choice = rng.random_range(0..3);
        match choice {
            0 => {
                let i = rng.random_range(0..TEST_SIZE);
                ops.push(Op::Insert(i));
            }
            1 => {
                let a = rng.random_range(0..TEST_SIZE);
                let b = rng.random_range(0..TEST_SIZE);
                ops.push(Op::Connect(a, b));
            }
            _ => {
                let i = rng.random_range(0..TEST_SIZE);
                ops.push(Op::DeleteGroup(i));
            }
        }
    }

    ops
}

fn run_ops<T: IdxTrait>(ops: &[Op]) {
    let mut ds = DisjointSet::<T>::new(TEST_SIZE);

    for op in ops {
        match *op {
            Op::Insert(i) => {
                ds.insert(i);
            }
            Op::Connect(a, b) => {
                ds.connect(a, b);
            }
            Op::DeleteGroup(i) => {
                ds.delete_group(i);
            }
        }
    }
}

const TEST_SIZE: usize = 19 * 19;

fn bench_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("disjoint_set_compare");

    // Generate once (outside benchmark timing)
    let ops = generate_ops(10000, 114514);

    group.bench_function("usize", |b| {
        b.iter(|| {
            run_ops::<usize>(black_box(&ops));
        })
    });
    group.bench_function("u32", |b| {
        b.iter(|| {
            run_ops::<u32>(black_box(&ops));
        })
    });
    group.bench_function("u16", |b| {
        b.iter(|| {
            run_ops::<u16>(black_box(&ops));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_all);
criterion_main!(benches);
