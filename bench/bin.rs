#![crate_id = "sort-bench"]
#![crate_type = "bin"]

extern crate debug;
extern crate test;
extern crate sort;
extern crate criterion;

//use criterion::Criterion;

fn main() {


  let ints = vec!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);

  let foo = 4;

  println!("{:?}", ints.as_slice().bsearch(|x| { foo.cmp(x) }));

  //let mut b = Criterion::new();

    //b.bench("exp", |b| {
        //let mut x: f64 = 2.0;
        //test::black_box(&mut x);

        //b.iter(|| x.exp())
    //});
}
