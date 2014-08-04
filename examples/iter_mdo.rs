#![feature(phase)]

#[phase(plugin, link)]
extern crate mdo;

fn main() {
    // exporting the monadic functions for the Iterator monad (similar
    // to list comprehension)
    use mdo::iter::{bind, ret, mzero};

    // getting the list of (x, y, z) such that
    //  - 1 <= x <= y < z < 11
    //  - x^2 + y^2 == z^2
    let mut l = bind(range(1i, 11),
                     |z| bind(range(1, z + 1),
                              |y| bind(range(1, y + 1),
                                       |x| bind(if x * x + y * y == z * z { ret(()) }
                                                else { mzero() },
                                                |_| ret((x, y, z))))));
    println!("{}", l.collect::<Vec<(int, int, int)>>());

    // the same thing, using the mdo! macro
    let l = mdo! {
        bind z = range(1i, 11);
        bind x = range(1, z);
        bind y = range(x, z);
        when x * x + y * y == z * z;
        ret((x, y, z))
    }.collect::<Vec<(int, int, int)>>();
    println!("{}", l);
}
