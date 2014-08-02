extern crate criterion;
extern crate patience_sort;


use std::rand;
use std::rand::Rng;

use patience_sort::patience_sort;

#[allow(dead_code)]
fn main() {
    let mut rng = rand::task_rng();
    let n = 100u;

    loop {
      patience_sort(rng.gen_iter::<u8>().take(n).collect::<Vec<u8>>().as_mut_slice(), |a, b| a.cmp(b));
    }
}
