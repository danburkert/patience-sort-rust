#![feature(phase)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate quickcheck;
extern crate ringbuf;

use std::ptr;
use std::mem;
use ringbuf::RingBuf;
use std::collections::Deque;

use std::iter::Peekable;

pub fn patience_sort<T : std::fmt::Show>(slice: &mut [T], cmp: |&T, &T| -> Ordering) {
    let len = slice.len();
    if len <= 1 { return; }
    let mut runs: Vec<RingBuf<T>> = generate_runs(slice, |a, b| cmp(a, b));

    unbalanced_ping_pong_merge(slice.as_mut_ptr(), slice.len(), runs, |a, b| cmp(a, b));
}

#[deriving(PartialEq, Eq)]
enum BufferSelector { A, B }
type Offset = uint;
type Length = uint;

/// Copy sorted runs into a buffer in ascending length order.  The buffer must
/// be large enough to hold every element.
unsafe fn copy_runs_into_buf<T : std::fmt::Show>(buf: *mut T,
                                                 mut runs: Vec<RingBuf<T>>)
                                                 -> Vec<(BufferSelector, Offset, Length)> {
    runs.sort_by(|a, b| a.len().cmp(&b.len()));
    let mut offset = 0u;
    let mut indices = Vec::with_capacity(runs.len());
    for mut run in runs.move_iter() {
        {
            let (slice1, slice2) = run.as_slices();
            ptr::copy_nonoverlapping_memory(buf.offset(offset as int),
                                            slice1.as_ptr(),
                                            slice1.len());
            ptr::copy_nonoverlapping_memory(buf.offset((offset + slice1.len()) as int),
                                            slice2.as_ptr(),
                                            slice2.len());
            indices.push((A, offset, run.len()));
            offset += run.len();
        }

        // Don't drop copied values
        run.set_len(0);
    }
    indices
}

fn unbalanced_ping_pong_merge<T : std::fmt::Show>(a: *mut T, len: uint,
                                                      runs: Vec<RingBuf<T>>,
                                                      cmp: |&T, &T| -> Ordering) {
    let mut run_bufs: Vec<(BufferSelector, uint, uint)> =  unsafe { copy_runs_into_buf(a, runs) };

    let mut _b = Vec::with_capacity(len);
    unsafe { _b.set_len(len); }
    let b = _b.as_mut_ptr();

    let mut current_run_index = 0u;
    while run_bufs.len() >= 2 {
        if run_bufs.len() == current_run_index + 1 {
            current_run_index = 0u;
            continue;
        }

        let (current_buffer, current_offset, current_len) = run_bufs[current_run_index];
        let (next_buffer, next_offset, next_len) = run_bufs[current_run_index + 1];
        let (_, _, first_len) = run_bufs[0];
        let (_, _, second_len) = run_bufs[1];
        let total_len = current_len + next_len;

        if first_len + second_len > total_len {
            current_run_index = 0u;
            continue;
        }

        unsafe {
            let next =
            match next_buffer {
                A => a.offset(next_offset as int),
                B => b.offset(next_offset as int)
            };

            let replacement_buffer = match current_buffer {
                A => {
                    blind_merge(a.offset(current_offset as int) as *const T, current_len,
                                next as *const T, next_len,
                                b.offset(current_offset as int), |a, b| cmp(a, b));
                    B
                }
                B => {
                    blind_merge(b.offset(current_offset as int) as *const T, current_len,
                                next as *const T, next_len,
                                a.offset(current_offset as int), |a, b| cmp(a, b));
                    A
                }
            };

            run_bufs.remove(current_run_index);
            run_bufs.push((replacement_buffer, current_offset, total_len));
            run_bufs.swap_remove(current_run_index);
        }
    }
    if run_bufs[0].val0() == B {
        unsafe { ptr::copy_nonoverlapping_memory(a, b as *const T, len); }
    }
}

/// Merge two sorted runs contained in the source buffers into the sink buffer.
///
/// The sink buffer must be large enough to hold `src1` and `src2`.
///
/// `src1` may not overlap with `sink`.
///
/// `src2` may overlap with `sink`, but only if `src2` is aligned such that the
/// last slot of `src2` is the last slot of `sink`.
///
/// `sink` should contain uninitialized slots, except where `src2` overlaps.
///
/// After returning, `src1` will contain uninitialized slots.  `src2` will
/// contain unitialized slots, unless it overlaps with `sink`.
unsafe fn blind_merge<T>(src1: *const T, src1_len: uint,
                         src2: *const T, src2_len: uint,
                         sink: *mut T, compare: |&T, &T| -> Ordering) {
    let sink_len = src1_len + src2_len;

    let mut idx1 = 0u; // index into src1
    let mut idx2 = 0u; // index into src2

    for i in range(0, sink_len) {
        debug_assert!(idx1 <= src1_len,
                      "Illegal index into src1: {}, src1 length: {}.", idx1, src1_len);
        debug_assert!(idx2 <= src2_len,
                      "Illegal index into src2: {}, src2 length: {}.", idx2, src2_len);

        if idx1 == src1_len {
            // src1 is exhausted; check if src2 overlaps sink
            let src_elem = src2.offset(idx2 as int);
            let sink_elem = sink.offset(i as int);
            if src_elem.to_uint() == sink_elem.to_uint() {
                break;
            } else {
                // copy the remaining elements from src2
                ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
                break;
            }
        }

        if idx2 == src2_len {
            // src2 is exhausted; copy the remaining elements from src1
            let src_elem = src1.offset(idx1 as int);
            let sink_elem = sink.offset(i as int);
            ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
            break;
        }

        let elem1 = src1.offset(idx1 as int);
        let elem2 = src2.offset(idx2 as int);
        let sink_elem = sink.offset(i as int);

        if compare(mem::transmute(elem1), mem::transmute(elem2)) == Less {
            ptr::copy_nonoverlapping_memory(sink_elem, elem1, 1);
            idx1 = idx1 + 1;
        } else {
            ptr::copy_nonoverlapping_memory(sink_elem, elem2, 1);
            idx2 = idx2 + 1;
        }
    }
}

/// Search a sorted slice for the index where the provided comparator returns
/// `Equal`.  If no such slot exists, the slot where an equal element would be
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

    let mut size = sqrt(slice.len());
    size += sqrt(size);
    let size_step = if slice.len() == 0 { 0 } else { size / slice.len() };

    let mut runs: Vec<RingBuf<T>> = Vec::with_capacity(size);

    for element in slice.iter() {
        // unwrap is safe, because we never insert an empty ringbuf into the runs vec
        let tail_index = bsearch(runs.as_slice(), |run| cmp(run.back().unwrap(), element));
        let val: T = unsafe { ptr::read(element) };
        if tail_index < runs.len() {
            runs.get_mut(tail_index).push(val);
        } else {
            // unwrap is safe, because we never insert an empty ringbuf into the runs vec
            let head_index = bsearch(runs.as_slice(), |run| cmp(element, run.front().unwrap()));
            if head_index < runs.len() {
                runs.get_mut(head_index).push_front(val);
            } else {
                let mut run = RingBuf::with_capacity(size);
                run.push(val);
                runs.push(run);
                size -= size_step;
            }
        }
    }
    runs
}

#[inline(always)]
pub fn sqrt(size: uint) -> uint{
    (size as f64).sqrt() as uint
}

mod check {
    #[phase(plugin)]
    extern crate quickcheck_macros;
    extern crate quickcheck;

    use super::patience_sort;
    use super::blind_merge;
    use super::generate_runs;
    use std::collections::Deque;
    use std::mem;

    #[test]
    fn test_generate_runs() {
        let input: Vec<uint> = vec![3u, 5, 4, 2, 1, 7, 6, 8, 9, 10];

        let expected: Vec<Vec<uint>> = vec![vec![1u, 2, 3, 5, 7, 8, 9, 10], vec![4u, 6]];

        let result = generate_runs(input.as_slice(), |&a, &b| a.cmp(&b));

        assert_eq!(expected, result.iter().map(|run| run.clone().into_vec()).collect());
    }

    #[test]
    fn test_foo() {
        let mut input: Vec<uint> = vec![3u, 4, 5, 6, 7, 8, 10, 13, 2, 1];
        let mut expected = input.clone();

        expected.sort();
        patience_sort(input.as_mut_slice(), |&a, &b| a.cmp(&b));

        println!("expected: {}", expected);
        println!("actual:   {}", input);

        assert!(input == expected)
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

    #[quickcheck]
    fn check_patience_sort(mut vec: Vec<int>) -> bool {
        let mut sorted_copy = vec.clone();
        sorted_copy.sort();
        patience_sort(vec.as_mut_slice(), |a, b| a.cmp(b));

        sorted_copy == vec
    }


    #[quickcheck]
    fn check_blind_merge(mut a: Vec<int>, mut b: Vec<int>) -> bool {
        let mut expected = Vec::new();
        expected.push_all(a.as_slice());
        expected.push_all(b.as_slice());
        expected.sort();

        a.sort();
        b.sort();

        let mut buffer = Vec::from_fn(a.len() + b.len(), |_| 0);

        unsafe {
            blind_merge(a.as_ptr(), a.len(),
                        b.as_ptr(), b.len(),
                        buffer.as_mut_ptr(),
                        |a, b| a.cmp(b));
        }
        expected == buffer
    }

    #[quickcheck]
    fn check_blind_merge_overlapping(mut a: Vec<int>, mut b: Vec<int>) -> bool {
        a.sort();
        b.sort();

        let mut buffer = Vec::from_fn(a.len(), |_| 0);
        buffer.push_all(b.as_slice());

        let mut expected = Vec::new();
        expected.push_all(a.as_slice());
        expected.push_all(b.as_slice());
        expected.sort();

        unsafe {
            blind_merge(a.as_ptr(), a.len(),
                        buffer.as_ptr().offset(a.len() as int), b.len(),
                        buffer.as_mut_ptr(),
                        |a, b| a.cmp(b));
        }
        expected == buffer
    }
}
