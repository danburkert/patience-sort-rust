#![crate_type = "lib"]
#![crate_id = "sort#0.1"]

extern crate debug;
extern crate collections;

extern crate quickcheck;

#[test]
extern crate quickcheck;
#[test]
extern crate debug;

use std::ptr;
use std::collections::ringbuf::RingBuf;
use std::collections::Deque;

fn patience_sort<T>(slice: &mut [T], compare: |&T, &T| -> Ordering) {
  let len = slice.len();
  let num_piles = (len as f64).sqrt() as uint;
  let mut piles: Vec<RingBuf<T>> = Vec::with_capacity(num_piles);

  for elem in slice.iter() {
    let pile_opt = piles.iter().find(|pile| compare(pile.back().unwrap(), elem) == Less);
    match pile_opt {
      None => {
        let mut rb = RingBuf::new();
        let raw: *T = elem;
        let r: T = unsafe { *raw };
        unsafe { rb.push_back(*raw); }
        piles.push(rb);
      }
      // Todo match on a real pile, and add the new element to it
      // match on None and create a new pile with the element.
      Some(pile) => ()


    }
  }
}

fn ping_pong_merge<T>(runs: &mut [T], indices: &[uint]) {

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
        let src_elem = buf2.offset(idx2) as *T;
        let sink_elem = sink_buf.offset(i);
        ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
      }
      break;
    }

    if idx2 == src2_len {
      // src2 is exhausted; copy the remaining elements from src1
      unsafe {
        let src_elem = buf1.offset(idx1) as *T;
        let sink_elem = sink_buf.offset(i);
        ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
      }
      break;
    }

    unsafe {
      let elem1 = buf1.offset(idx1) as *T;
      let elem2 = buf2.offset(idx2) as *T;
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

/// Merge two sorted runs. The first run is contained in the `src1` slice.
/// The second run is contained in the `sink` slice offset by the length
/// of `src1`. The merged results will be stored in `sink`.
fn blind_slice_merge<T>(src1: &[T],
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
        let src_elem = buf1.offset(idx1) as *T;
        let sink_elem = sink_buf.offset(i);
        ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
      }
      break;
    }

    unsafe {
      let elem1 = buf1.offset(idx1) as *T;
      let elem2 = buf2.offset(idx2) as *T;
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

mod test {

  extern crate quickcheck;

  use quickcheck::quickcheck;
  use super::slice_merge;
  use super::blind_slice_merge;

  #[test]
  fn check_slice_merge() {

    fn prop(mut run1: Vec<int>, mut run2: Vec<int>) -> bool {
      run1.sort();
      run2.sort();

      let mut expected = run1.clone().append(run2.clone().as_slice());
      expected.sort();

      let mut sink = Vec::from_elem(expected.len(), 0i);

      slice_merge(run1.as_slice(), run2.as_slice(), sink.as_mut_slice(), |a,b| a.cmp(b));

      expected == sink
    }

    quickcheck(prop);
  }

  #[test]
  fn check_blind_slice_merge() {

    fn prop(mut run1: Vec<int>, mut run2: Vec<int>) -> bool {
      run1.sort();
      run2.sort();

      let mut sink = Vec::from_elem(run1.len(), 0i).append(run2.as_slice());

      let mut expected = run1.clone().append(run2.clone().as_slice());
      expected.sort();

      blind_slice_merge(run1.as_slice(), sink.as_mut_slice(), |a,b| a.cmp(b));

      expected == sink
    }

    quickcheck(prop);
  }
}
