
//use collections::dlist::DList;

//fn patience_sort<T>(v: &mut Vec<T>, compare: |&T, &T| -> Ordering) {
  //let len: uint = v.len();

  //let initial_runs = (len as f64).sqrt() as uint;
  //let mut runs: Vec<DList<T>> = Vec::with_capacity(initial_runs);

//}

fn blind_merge<T : Ord>(slice: &mut [T], index: uint) {
  let len = slice.len();
  let mut i = 0u;
  let mut j = index;

  loop {
    if i == len - 1 { break; }
    if i == j { j = j + 1; }

    if slice[i] > slice[j] {
      slice.swap(i, j);
    }

    i = i + 1;
  }

  return;
}

mod test {
  use super::blind_merge;

  //#[test]
  //fn test_patience_sort() {

    //println!("Hello, world!");
    //assert!(true);
  //}

  #[test]
  fn test_blind_merge() {
    let mut ints = vec!(1, 3, 5, 2, 4, 6);
    let slice = ints.as_mut_slice();

    blind_merge(slice, 3);

    assert!(slice == vec!(1, 2, 3, 4, 5, 6).as_slice());
  }

}
