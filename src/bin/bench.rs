extern crate criterion;
extern crate patience_sort;


use std::rand::Rng;
use std::rand::SeedableRng;
use std::rand::StdRng;

use criterion::{Bencher, Criterion};

use patience_sort::patience_sort;

#[allow(dead_code)]
fn main() {
    let mut b = Criterion::default();
    let sizes = &[8u, 1024];

    b.bench_family("patience_sort_uniform", patience_sort_uniform, sizes);
    //b.bench_family("std_sort_uniform", std_sort_uniform, sizes);
    b.bench_family("patience_sort_sorted", patience_sort_sorted, sizes);
    //b.bench_family("std_sort_sorted", std_sort_sorted, sizes);
}

#[inline]
fn get_rng() -> StdRng {
    SeedableRng::from_seed(&[1u, 2, 3, 4])
}

#[allow(dead_code)]
fn patience_sort_uniform(b: &mut Bencher, size: &uint) {
    let items: Vec<int> = get_rng().gen_iter::<int>().take(*size).collect();
    b.iter(|| {
        patience_sort(items.clone().as_mut_slice(), |a, b| a.cmp(b));
    })
}

#[allow(dead_code)]
fn std_sort_uniform(b: &mut Bencher, size: &uint) {
    let items: Vec<int> = get_rng().gen_iter::<int>().take(*size).collect();
    b.iter(|| {
        items.clone().sort();
    })
}

#[allow(dead_code)]
fn patience_sort_sorted(b: &mut Bencher, size: &uint) {
    let items: Vec<int> = Vec::from_fn(*size, |i| i as int);
    b.iter(|| {
        patience_sort(items.clone().as_mut_slice(), |a, b| a.cmp(b));
    })
}

#[allow(dead_code)]
fn std_sort_sorted(b: &mut Bencher, size: &uint) {
    let items: Vec<int> = Vec::from_fn(*size, |i| i as int);
    b.iter(|| {
        items.clone().sort();
    })
}
