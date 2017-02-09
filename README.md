# Rust mdo [![Build Status](https://travis-ci.org/TeXitoi/rust-mdo.svg?branch=master)](https://travis-ci.org/TeXitoi/rust-mdo)

## Presentation

Rust mdo is a monadic do notation using macro and duck typing.  It
provides a syntax extention providing something looking like the
Haskell do notation, and rewrite it using a `bind` function.  Some
functions are privided for some common monadic structures.

## Example

Add to your `Cargo.toml`:
```toml
[dependencies]
mdo = "*"
```

```rust
#[macro_use] extern crate mdo;

fn main() {
    // exporting the monadic functions for the Iterator monad (similar
    // to list comprehension)
    use mdo::iter::{bind, ret, mzero};

    // getting the list of (x, y, z) such that
    //  - 1 <= x <= y < z < 11
    //  - x^2 + y^2 == z^2
    let l = bind(1i32..11, move |z|
                 bind(1..z, move |x|
                      bind(x..z, move |y|
                           bind(if x * x + y * y == z * z { ret(()) }
                                else { mzero() },
                                move |_|
                                ret((x, y, z))
                                )))).collect::<Vec<_>>();
    println!("{:?}", l);

    // the same thing, using the mdo! macro
    let l = mdo! {
        z =<< 1i32..11;
        x =<< 1..z;
        y =<< x..z;
        when x * x + y * y == z * z;
        ret ret((x, y, z))
    }.collect::<Vec<_>>();
    println!("{:?}", l);
}
```

## Documentation

You can find the rustdoc [here](https://texitoi.eu/rust/mdo/).

## Supported monad

This library provides the definitions for the following monads:
  - `Option`
  - `Iterator`
  - `Result` (without error conversion)
  
Other implemetation in external crates:
  - `Future` in [mdo-future](https://crates.io/crates/mdo-future)

## License

This work is free. You can redistribute it and/or modify it under the
terms of the Do What The Fuck You Want To Public License, Version 2,
as published by Sam Hocevar. See the COPYING file for more details.
