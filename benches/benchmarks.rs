use std::time::Instant;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

extern crate indexlist;
use indexlist::IndexList;
use rand::{distributions::Uniform, prelude::Distribution};

fn index_list_insert_benchmark(c: &mut Criterion) {
    let max = 10;
    let size = 100;
    let iterations = 5;
    let mut group_insert = c.benchmark_group("insert");

    let mut rng = rand::thread_rng();
    let between = Uniform::from(0..max);

    for elem in between.sample_iter(&mut rng).take(iterations) {
        let mut index_list = IndexList::with_capacity(max);
        let mut indexes = Vec::with_capacity(size);
        for _ in 0..size {
            indexes.push(index_list.push_back(0));
        }

        group_insert.bench_with_input(
            BenchmarkId::from_parameter(format!("IndexList-{}-{:?}", elem, Instant::now())),
            &elem,
            |b, &elem| {
                b.iter(|| index_list.insert_after(indexes[elem], elem));
            },
        );
    }

    for elem in between.sample_iter(&mut rng).take(iterations) {
        let mut vector = Vec::with_capacity(size);
        for _ in 0..size {
            vector.push(0);
        }

        group_insert.bench_with_input(
            BenchmarkId::from_parameter(format!("Vector-{}-{:?}", elem, Instant::now())),
            &elem,
            |b, &elem| {
                b.iter(|| vector.insert(elem, elem));
            },
        );
    }

    group_insert.finish();
}

fn index_list_remove_benchmark(c: &mut Criterion) {
    let max = 10;
    let size = 1_000_000;
    let iterations = 5;
    let mut group_remove = c.benchmark_group("remove");

    let mut rng = rand::thread_rng();
    let between = Uniform::from(0..max);

    for elem in between.sample_iter(&mut rng).take(iterations) {
        let mut index_list = IndexList::with_capacity(size);
        for _ in 0..size {
            index_list.push_back(0);
        }

        group_remove.bench_with_input(
            BenchmarkId::from_parameter(format!("IndexList-{}-{:?}", elem, Instant::now())),
            &elem,
            |b, &_| {
                b.iter(|| {
                    if let Some(index) = index_list.head_index() {
                        index_list.remove(index);
                    }
                });
            },
        );
    }

    for elem in between.sample_iter(&mut rng).take(iterations) {
        let mut vector = Vec::with_capacity(size);
        for _ in 0..size {
            vector.push(0);
        }

        group_remove.bench_with_input(
            BenchmarkId::from_parameter(format!("Vector-{}-{:?}", elem, Instant::now())),
            &elem,
            |b, &_| {
                b.iter(|| {
                    if !vector.is_empty() {
                        vector.remove(0);
                    }
                });
            },
        );
    }

    group_remove.finish();
}

criterion_group!(
    benches,
    index_list_insert_benchmark,
    index_list_remove_benchmark,
);
criterion_main!(benches);
