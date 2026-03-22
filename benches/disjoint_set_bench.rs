use criterion::{Criterion, criterion_group, criterion_main};
use rustgo::DisjointSet;
use std::hint::black_box;

const TEST_SIZE: usize = 10240;

fn bench_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("disjoint_set_compare");

    group.bench_function("usize", |b| {
        b.iter(|| {
            let mut ds = DisjointSet::<usize>::new(black_box(TEST_SIZE));
            for i in (0..black_box(TEST_SIZE)).step_by(7) {
                ds.insert(i);
            }
            for i in (0..black_box(TEST_SIZE)).step_by(3) {
                ds.connect(i, i / 7);
            }
            for i in (0..black_box(TEST_SIZE)).step_by(2) {
                ds.delete_group(i);
            }
        })
    });

    group.bench_function("u32", |b| {
        b.iter(|| {
            let mut ds = DisjointSet::<u32>::new(black_box(TEST_SIZE));
            for i in (0..black_box(TEST_SIZE)).step_by(7) {
                ds.insert(i);
            }
            for i in (0..black_box(TEST_SIZE)).step_by(3) {
                ds.connect(i, i / 7);
            }
            for i in (0..black_box(TEST_SIZE)).step_by(2) {
                ds.delete_group(i);
            }
        })
    });

    group.bench_function("u16", |b| {
        b.iter(|| {
            let mut ds = DisjointSet::<u16>::new(black_box(TEST_SIZE));
            for i in (0..black_box(TEST_SIZE)).step_by(7) {
                ds.insert(i);
            }
            for i in (0..black_box(TEST_SIZE)).step_by(3) {
                ds.connect(i, i / 7);
            }
            for i in (0..black_box(TEST_SIZE)).step_by(2) {
                ds.delete_group(i);
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_all);
criterion_main!(benches);
