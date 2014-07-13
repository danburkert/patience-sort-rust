#![crate_type = "lib"]
#![crate_name = "sort"]

use std::ptr;

pub fn patience_sort<T : std::fmt::Show>(slice: &mut [T], cmp: |&T, &T| -> Ordering) {
    let mut runs: Vec<Vec<T>> = Vec::new();

    for element in slice.iter() {
        let index = runs_bsearch(&runs, element, |a, b| cmp(a, b));
        let val: T = unsafe { ptr::read(element) };
        if index == runs.len() {
            runs.push(vec![val]);
        } else {
           runs.get_mut(index).push(val)
        }
    }

    let mut offset = 0u;
    for mut run in runs.move_iter() {
        unsafe {
        let sub_slice = slice.mut_slice_from(offset);
            sub_slice.copy_memory(run.as_mut_slice());
            offset += run.len();
            run.set_len(0);
        }
    }
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
