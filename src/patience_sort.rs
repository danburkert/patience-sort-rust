#![crate_type = "lib"]

#![feature(phase)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate quickcheck;

use std::ptr;
use std::mem;

pub fn patience_sort<T : std::fmt::Show>(slice: &mut [T], cmp: |&T, &T| -> Ordering) {
    let len = slice.len();
    if len == 0 { return; }
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

    // Do the initial merge back into the original slice.
    let run1 = runs.pop().unwrap(); // Safe because `len > 0`
    let run2 = runs.pop().unwrap_or(Vec::new());
    let src1 = run1.as_slice();
    let src2 = run2.as_slice();
    let mut idx = len - src1.len() - src2.len();
    { // necessary for borrow scope
        let dest = slice.mut_slice_from(idx);
        slice_merge(src1, src2, dest, |a, b| cmp(a, b));
    }

    // Do subsequent blind merges
    for run in runs.iter().rev() {
        idx -= run.len();
        let dest = slice.mut_slice_from(idx);
        blind_merge(run.as_slice(), dest, |a, b| cmp(a, b));
    }
}

/// Merge two sorted runs contained in the `source` slices into the `sink` slice.
/// The slices may not overlap. The length of the `sink` slice must equal to the
/// sum of the `source` slice lengths.
fn slice_merge<T>(src1: &[T],
                  src2: &[T],
                  sink: &mut [T],
                  compare: |&T, &T| -> Ordering) {
    let src1_len = src1.len() as int;
    let src2_len = src2.len() as int;
    let sink_len = sink.len() as int;
    debug_assert!(src1_len + src2_len == sink_len,
                  "Illegal slice lengths. src1 len: {}. src2 len: {}. sink len: {}.",
                  src1_len, src2_len, sink_len);

    let buf1 = src1.as_ptr();
    let buf2 = src2.as_ptr();
    let sink_buf = sink.as_mut_ptr();

    let mut idx1 = 0i; // index into src1 slice
    let mut idx2 = 0i; // index into src2 slice

    for i in range(0, sink_len) {
        debug_assert!(idx1 >= 0 && idx1 <= src1_len,
                      "Illegal index into src1: {}, src1 length: {}.", idx1, src1_len);
        debug_assert!(idx2 >= 0 && idx2 <= src2_len,
                      "Illegal index into src2: {}, src2 length: {}.", idx2, src2_len);
        debug_assert!(i >= 0 && i < sink_len,
                      "Illegal index into sink: {}, sink length: {}.", i, sink_len);

        if idx1 == src1_len {
            // src1 is exhausted; copy the remaining elements from src2
            unsafe {
                let src_elem = buf2.offset(idx2) as *const T;
                let sink_elem = sink_buf.offset(i);
                ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
            }
            break;
        }

        if idx2 == src2_len {
            // src2 is exhausted; copy the remaining elements from src1
            unsafe {
                let src_elem = buf1.offset(idx1) as *const T;
                let sink_elem = sink_buf.offset(i);
                ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
            }
            break;
        }

        unsafe {
            let elem1 = buf1.offset(idx1) as *const T;
            let elem2 = buf2.offset(idx2) as *const T;
            let sink_elem = sink_buf.offset(i);

            if compare(mem::transmute(elem1), mem::transmute(elem2)) == Less {
                ptr::copy_nonoverlapping_memory(sink_elem, elem1, 1);
                idx1 = idx1 + 1;
            } else {
                ptr::copy_nonoverlapping_memory(sink_elem, elem2, 1);
                idx2 = idx2 + 1;
            }
        }
    }
}

/// Merge two sorted runs. The first run is contained in the `src1` slice.
/// The second run is contained in the `sink` slice offset by the length
/// of `src1`. The merged results will be stored in `sink`.
fn blind_merge<T>(src1: &[T],
                  sink: &mut [T],
                  compare: |&T, &T| -> Ordering) {
    let sink_len = sink.len() as int;
    let src1_len = src1.len() as int;
    let src2_len = sink_len - src1_len;
    debug_assert!(src1.len() <= sink.len(),
    "Illegal slice lengths. src1 len: {}. sink len: {}.", src1_len, sink_len);

    let buf1 = src1.as_ptr();
    let buf2 = unsafe { sink.as_ptr().offset(src1_len) };
    let sink_buf = sink.as_mut_ptr();

    let mut idx1 = 0i; // index into src1 slice
    let mut idx2 = 0i; // index into src2 slice

    for i in range(0, sink_len) {
        debug_assert!(idx1 >= 0 && idx1 <= src1_len,
                      "Illegal index into src1: {}, src1 length: {}.", idx1, src1_len);
        debug_assert!(idx2 >= 0 && idx2 <= src2_len,
                      "Illegal index into src2: {}, src2 length: {}.", idx2, src2_len);
        debug_assert!(i >= 0 && i < sink_len,
                      "Illegal index into sink: {}, sink length: {}.", i, sink_len);

        if idx1 == src1_len {
            // src1 is exhausted;
            break;
        }

        if idx2 == src2_len {
            // src2 is exhausted; copy the remaining elements from src1
            unsafe {
                let src_elem = buf1.offset(idx1) as *const T;
                let sink_elem = sink_buf.offset(i);
                ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
            }
            break;
        }

        unsafe {
            let elem1 = buf1.offset(idx1) as *const T;
            let elem2 = buf2.offset(idx2) as *const T;
            let sink_elem = sink_buf.offset(i);

            if compare(&*elem1, &*elem2) == Less {
                ptr::copy_nonoverlapping_memory(sink_elem, elem1, 1);
                idx1 = idx1 + 1;
            } else {
                ptr::copy_nonoverlapping_memory(sink_elem, elem2, 1);
                idx2 = idx2 + 1;
            }
        }
    }
}

#[inline]
fn runs_search<'a, T>(runs: &'a Vec<Vec<T>>,
                      element: &T,
                      cmp: |&T, &T| -> Ordering)
                      -> Option<&'a Vec<T>> {
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
                   cmp: |&T, &T| -> Ordering)
                   -> uint {
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
    #[phase(plugin)]
    extern crate quickcheck_macros;
    extern crate quickcheck;

    use super::patience_sort;
    use super::runs_search;
    use super::runs_bsearch;
    use super::slice_merge;
    use super::blind_merge;

    #[quickcheck]
    fn check_patience_sort(mut vec: Vec<int>) -> bool {
        let mut sorted_copy = vec.clone();
        sorted_copy.sort();
        patience_sort(vec.as_mut_slice(), |a, b| a.cmp(b));

        sorted_copy == vec
    }


    #[quickcheck]
    fn check_slice_merge(mut a: Vec<int>, mut b: Vec<int>) -> bool {
        let mut expected = Vec::new();
        expected.push_all(a.as_slice());
        expected.push_all(b.as_slice());
        expected.sort();

        a.sort();
        b.sort();

        let mut buffer = Vec::from_fn(a.len() + b.len(), |_| 0);

        slice_merge(a.as_slice(), b.as_slice(), buffer.as_mut_slice(), |a, b| a.cmp(b));
        expected == buffer
    }

    #[quickcheck]
    fn check_blind_slice_merge(mut a: Vec<int>, mut b: Vec<int>) -> bool {
        a.sort();
        b.sort();

        let mut buffer = Vec::from_fn(a.len(), |_| 0);
        buffer.push_all(b.as_slice());

        let mut expected = Vec::new();
        expected.push_all(a.as_slice());
        expected.push_all(b.as_slice());
        expected.sort();

        blind_merge(a.as_slice(), buffer.as_mut_slice(), |a, b| a.cmp(b));

        expected == buffer
    }

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
}
