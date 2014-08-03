# Rust do

## Presentation

Rust do is a monadic do notation using macro and duck typing.  It
provides a syntax extention providing something looking like the
Haskell do notation, and rewrite it using a bind function.  Some
functions are privided for some common monadic structures.

## Example

```rust
#![feature(phase)]

#[phase(plugin, link)]
extern crate mdo;

fn main() {
    use mdo::iter::{bind, guard, ret};

    let mut l = bind(range(1i, 11),
                     |z| bind(range(1, z + 1),
                              |y| bind(range(1, y + 1),
                                       |x| bind(guard(x * x + y * y == z * z),
                                                |_| ret((x, y, z))))));
    assert_eq!(l.collect::<Vec<(int, int, int)>>(), vec![(3, 4, 5), (6, 8, 10)]);

    let l = mdo! {
        | z <- range(1i, 11);
        | y <- range(1, z);
        | x <- range(1, y + 1);
        + let test = x * x + y * y == z * z;
        | _ <- guard(test);
        > { let res = (x, y, z); ret(res) }
    }.collect::<Vec<(int, int, int)>>();
    assert_eq!(l, vec![(3, 4, 5), (6, 8, 10)]);
}
```
