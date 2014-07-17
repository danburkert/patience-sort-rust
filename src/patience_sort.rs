#![crate_type = "lib"]

#![feature(phase)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate quickcheck;
extern crate ringbuf;

use std::ptr;
use std::mem;
use ringbuf::RingBuf;
use std::collections::Deque;

pub fn patience_sort<T : std::fmt::Show>(slice: &mut [T], cmp: |&T, &T| -> Ordering) {
    let len = slice.len();
    if len <= 1 { return; }
    let mut runs: Vec<RingBuf<T>> = generate_runs(slice, cmp);

    // Do the initial merge back into the original slice.
    //let run1 = runs.pop().unwrap(); // Safe because `len > 0`
    //let run2 = runs.pop().unwrap_or(RingBuf::new());
    //let src1 = run1.into_vec().as_slice();
    //let src2 = run2.into_vec().as_slice();
    //let mut idx = len - src1.len() - src2.len();
    { // necessary for borrow scope
        //let dest = slice.mut_slice_from(idx);
        //slice_merge(src1, src2, dest, |a, b| cmp(a, b));
    }

    // Do subsequent blind merges
    for run in runs.iter().rev() {
        //idx -= run.len();
        //let dest = slice.mut_slice_from(idx);
        //blind_merge(run.as_slice(), dest, |a, b| cmp(a, b));
    }
}

/// Merge two sorted iterators into the `sink` slice.
/// The length of the iterators must equal the sink slice.
fn iterator_merge<T>(src1: &Iterator<T>,
                     src2: &Iterator<T>,
                     sink: &mut [T],
                     compare: |&T, &T| -> Ordering) {
    let sink_len = sink.len() as int;

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

/// Search a sorted slice for the index where the provided comparator returns
/// `Equal`.  If no such slot exists, the slot where the element would be
/// inserted is returned.  Note that this may be past the end of the slice.
fn bsearch<T>(slice: &[T], cmp: |&T| -> Ordering) -> uint {
    let mut base = 0u;
    let mut lim = slice.len();

    while lim != 0 {
        let ix = base + (lim >> 1);
        match cmp(&slice[ix]) {
            Less => (),
            Greater => {
                base = ix + 1;
                lim -= 1;
            },
            Equal => return ix
        }
        lim >>= 1;
    }
    return base;
}

fn generate_runs<T>(slice: &[T],
                    cmp: |&T, &T| -> Ordering)
                    -> Vec<RingBuf<T>> {
    let mut runs: Vec<RingBuf<T>> = Vec::with_capacity(sqrt(slice.len()));

    for element in slice.iter() {

        // unwrap is safe, because we never insert an empty run (ringbuf) into the runs vec
        let tail_index = bsearch(runs.as_slice(), |run| cmp(run.back().unwrap(), element));
        let val: T = unsafe { ptr::read(element) };
        if tail_index < runs.len() {
            runs.get_mut(tail_index).push_back(val);
        } else {
            // unwrap is safe, because we never insert an empty run (ringbuf) into the runs vec
            let head_index = bsearch(runs.as_slice(), |run| cmp(element, run.front().unwrap()));
            if head_index < runs.len() {
                runs.get_mut(head_index).push_front(val);
            } else {
                let mut run = RingBuf::new();
                run.push_back(val);
                runs.push(run);
            }
        }
    }

    runs
}

#[inline]
pub fn sqrt(size: uint) -> uint{
    (size as f64).sqrt() as uint
}

mod check {
    #[phase(plugin)]
    extern crate quickcheck_macros;
    extern crate quickcheck;

    use super::patience_sort;
    use super::slice_merge;
    use super::blind_merge;
    use super::generate_runs;
    use std::collections::Deque;

    #[test]
    fn test_generate_runs() {
        let input: Vec<uint> = vec![3u, 5, 4, 2, 1, 7, 6, 8, 9, 10];

        let expected: Vec<Vec<uint>> = vec![vec![1u, 2, 3, 5, 7, 8, 9, 10], vec![4u, 6]];

        let result = generate_runs(input.as_slice(), |&a, &b| a.cmp(&b));

        assert_eq!(expected, result.iter().map(|run| run.clone().into_vec()).collect());
    }

    #[quickcheck]
    fn check_generate_runs(vec: Vec<int>) -> bool {

        let runs = generate_runs(vec.as_slice(), |a, b| a.cmp(b));
        println!("");
        println!("vec: {}", vec);
        println!("runs: {}", runs);

        let heads: Vec<int> = runs.iter().map(|run| run.front().unwrap().clone()).collect();
        let mut sorted_heads = heads.clone();
        sorted_heads.sort();

        let tails: Vec<int> = runs.iter().map(|run| run.back().unwrap().clone()).collect();
        let mut sorted_tails = tails.clone();
        sorted_tails.sort();
        sorted_tails.reverse();

        let runs_are_sorted = runs.iter().all(|run| {
            let mut sorted_run = run.clone().into_vec();
            sorted_run.sort();
            sorted_run == run.clone().into_vec()
        });

        println!("heads: {}, sorted heads: {}", heads, sorted_heads);

        vec.len() >= runs.len() && runs_are_sorted && heads == sorted_heads && tails == sorted_tails
    }

    // #[quickcheck]
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
}
