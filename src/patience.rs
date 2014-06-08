#[test]
extern crate quickcheck;
#[test]
extern crate debug;

use std::ptr;

fn ping_pong_merge<T>(runs: &mut [T], indices: &[uint]) {

}

/// Merge two sorted `source` slices into a third `sink` slice. The slices may not overlap.
/// The length of the `sink` slice must equal to the sum of the `source` slice lengths.
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

  let mut index1 = 0i; // index into src1 slice
  let mut index2 = 0i; // index into src2 slice

  for i in range(0, sink_len) {
    debug_assert!(index1 >= 0 && index1 <= src1_len,
                  "Illegal index into src1: {}, src1 length: {}.", index1, src1_len);
    debug_assert!(index2 >= 0 && index2 <= src2_len,
                  "Illegal index into src2: {}, src2 length: {}.", index2, src2_len);
    debug_assert!(i >= 0 && i < sink_len,
                  "Illegal index into sink: {}, sink length: {}.", i, sink_len);

    if index1 == src1_len {
      // src1 is exhausted; copy the remaining elements from src2
      unsafe {
        let src_elem = buf2.offset(index2) as *T;
        let sink_elem = sink_buf.offset(i);
        ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
      }
      break;
    }

    if index2 == src2_len {
      // src2 is exhausted; copy the remaining elements from src1
      unsafe {
        let src_elem = buf1.offset(index1) as *T;
        let sink_elem = sink_buf.offset(i);
        ptr::copy_nonoverlapping_memory(sink_elem, src_elem, (sink_len - i) as uint);
      }
      break;
    }

    unsafe {
      let elem1 = buf1.offset(index1) as *T;
      let elem2 = buf2.offset(index2) as *T;
      let sink_elem = sink_buf.offset(i);

      if compare(&*elem1, &*elem2) == Less {
        ptr::copy_nonoverlapping_memory(sink_elem, elem1, 1);
        index1 = index1 + 1;
      } else {
        ptr::copy_nonoverlapping_memory(sink_elem, elem2, 1);
        index2 = index2 + 1;
      }
    }
  }
}

mod test {

  extern crate quickcheck;

  use quickcheck::quickcheck;
  use super::slice_merge;

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
}
