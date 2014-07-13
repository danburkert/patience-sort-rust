#![crate_type = "lib"]
#![crate_name = "sort"]

use std::ptr;
use std::iter::Peekable;

struct MergingIterator<'a, A, T, U> {
    a: Peekable<A, T>,
    b: Peekable<A, U>,
    f: |&A, &A|: 'a -> Ordering
}

enum IteratorSelector{ A, B }

impl<'a, A, T : Iterator<A>, U : Iterator<A>> Iterator<A> for MergingIterator<'a, A, T, U> {
    #[inline]
    fn next(&mut self) -> Option<A> {
        let mut a: &mut Peekable<A, T> = &mut self.a;
        let mut b: &mut Peekable<A, U> = &mut self.b;
        let mut f: &mut |&A, &A| -> Ordering = &mut self.f;

        match get_next(a.peek(), b.peek(), |a, b| (*f)(a, b)) {
            A => a.next(),
            B => b.next()
        }
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = a_lower + b_lower;
        let upper = a_upper.and_then(|a| b_upper.map(|b| a + b));
        (lower, upper)
    }
}

fn get_next<'a, A>(a: Option<&A>,
                   b: Option<&A>,
                   f: |&A, &A| -> Ordering)
                   -> IteratorSelector {
    match (a, b) {
        (None, _) => B,
        (_, None) => A,
        (Some(a_next), Some(b_next)) if f(a_next, b_next) == Greater => B,
        _ => A
    }
}

pub fn patience_sort<T : std::fmt::Show>(slice: &mut [T], cmp: |&T, &T| -> Ordering) {
    let mut runs: Vec<Vec<T>> = Vec::new();

    for element in slice.iter() {
        let index = runs_bsearch(&runs, element, |a, b| cmp(a, b));
        let val: T = unsafe { ptr::read(element) };
        if index == runs.len() {
            runs.push(vec![val]);
        } else {
           runs.get_mut(index).push(val);
        }
    }

    let mut offset = 0u;

    let runs_iter = runs.move_iter().rev();

    let first: std::vec::MoveItems<T> =  runs_iter.next().map_or(Vec::new().move_iter(), |run| run.move_iter());
    let second: std::vec::MoveItems<T> = runs_iter.next().map_or(Vec::new().move_iter(), |run| run.move_iter());
    let zero = MergingIterator { a: first.peekable(), b: second.peekable(), f: |a, b| cmp(a, b) };

    let merged = runs.move_iter().rev().fold(zero, |acc, vec| {
        MergingIterator { a: acc.peekable(), b: vec.move_iter().peekable(), f: |a, b| cmp(a, b) }
    });

    fail!();
}

#[inline]
fn runs_search<'a, T>(runs: &'a Vec<Vec<T>>,
                 element: &T,
                 cmp: |&T, &T| -> Ordering
                ) -> Option<&'a Vec<T>> {
    for run in runs.iter() {
        match run.last() {
            None => return Some(run),
            Some(last) => if cmp(element, last) != Less { return Some(run) }
        }
    }

    return None;
}

/// Search a sorted slice for the index where the provided element would be
/// inserted. The returned index will be in [0, runs.len()].
#[inline]
fn runs_bsearch<T>(runs: &Vec<Vec<T>>,
                   element: &T,
                   cmp: |&T, &T| -> Ordering) -> uint {
    let mut base = 0u;
    let mut lim = runs.len();

    while lim != 0 {
        let ix = base + (lim >> 1);
        match runs.get(ix).last() {
            None => (),
            Some(val) => {
                match cmp(val, element) {
                    Less => (),
                    Greater => {
                        base = ix + 1;
                        lim -= 1;
                    },
                    Equal => return ix
                }
            }
        }
        lim >>= 1;
    }
    return base;
}

mod check {
  use super::patience_sort;
  use super::runs_search;
  use super::runs_bsearch;

  #[test]
  fn test_runs_search() {
      let runs = vec![
          vec![3i, 5, 7, 8, 9, 10],
          vec![4, 6],
          vec![2],
          vec![1]];

      assert_eq!(Some(runs.get(0)), runs_search(&runs, &100i, |a, b| a.cmp(b)));
      assert_eq!(Some(runs.get(0)), runs_search(&runs, &10i, |a, b| a.cmp(b)));
      assert_eq!(Some(runs.get(1)), runs_search(&runs, &9i, |a, b| a.cmp(b)));
      assert_eq!(Some(runs.get(1)), runs_search(&runs, &6i, |a, b| a.cmp(b)));
      assert_eq!(Some(runs.get(2)), runs_search(&runs, &5i, |a, b| a.cmp(b)));
      assert_eq!(Some(runs.get(2)), runs_search(&runs, &2i, |a, b| a.cmp(b)));
      assert_eq!(Some(runs.get(3)), runs_search(&runs, &1i, |a, b| a.cmp(b)));
      assert_eq!(None, runs_search(&runs, &0i, |a, b| a.cmp(b)));
      assert_eq!(None, runs_search(&runs, &-1i, |a, b| a.cmp(b)));

  }

  #[test]
  fn test_search() {
      let runs = vec![
          vec![3i, 5, 7, 8, 9, 10],
          vec![4, 6],
          vec![2],
          vec![1]];

      assert_eq!(0, runs_bsearch(&runs, &100i, |a, b| a.cmp(b)));
      assert_eq!(0, runs_bsearch(&runs, &10i, |a, b| a.cmp(b)));
      assert_eq!(1, runs_bsearch(&runs, &9i, |a, b| a.cmp(b)));
      assert_eq!(1, runs_bsearch(&runs, &6i, |a, b| a.cmp(b)));
      assert_eq!(2, runs_bsearch(&runs, &5i, |a, b| a.cmp(b)));
      assert_eq!(2, runs_bsearch(&runs, &2i, |a, b| a.cmp(b)));
      assert_eq!(3, runs_bsearch(&runs, &1i, |a, b| a.cmp(b)));
      assert_eq!(4, runs_bsearch(&runs, &0i, |a, b| a.cmp(b)));
      assert_eq!(4, runs_bsearch(&runs, &-1i, |a, b| a.cmp(b)));

  }

  #[test]
  fn test_example() {
      let mut input = vec!(3i, 5, 4, 2, 1, 7, 6, 8, 9, 10);
      let expected = vec!(3i, 5, 7, 8, 9, 10, 4, 6, 2, 1);

      patience_sort(input.as_mut_slice(), |x, y| x.cmp(y));
      assert_eq!(expected, input);
  }
}
