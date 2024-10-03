#[macro_use]
extern crate criterion;

extern crate rand;

use rand::distributions::Uniform;
use rand::Rng;

extern crate indexlist;
use indexlist::IndexList;

extern crate generational_arena;
use generational_arena::Arena;

use std::collections::LinkedList;

use criterion::{Criterion, Fun};

fn criterion_benchmark(c: &mut Criterion) {}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
