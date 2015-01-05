// Copyright (c) 2014 Guillaume Pinot <texitoi(a)texitoi.eu>
//
// This work is free. You can redistribute it and/or modify it under
// the terms of the Do What The Fuck You Want To Public License,
// Version 2, as published by Sam Hocevar. See the COPYING file for
// more details.

#![deny(missing_docs)]
#![deny(warnings)]

#![feature(macro_rules)]

//! Monadic do notation

/// Monadic do notation using duck typing
///
/// Syntax:
/// `(instr)* ; ret expr`
///
/// instr can be:
///
/// * `pattern <- expression`: bind expression to pattern. a `bind`
///   function must be in scope.
///
/// * `let pattern = expression`: assign expression to pattern, as
///   normal rust let.
///
/// * `ign expression`: equivalent to `_ <- expression`
///
/// * `when expression`: filter on the monad. `ret` and `mzero`
///   functions must be in scope.
///
/// # Example
///
/// ```rust,ignore
/// use iter::{bind, ret, mzero};
/// let l = mdo! {
///     x <- range(0i, 5); // assign x to [0, 5[
///     ign range(0i, 2); // duplicate each value
///     when x % 2 == 0; // filter on even values
///     let y = x + 5; // create y
///     ret ret(y + 5) // return y + 5
/// }.collect::<Vec<int>>();
/// assert_eq!(l, vec![10, 10, 12, 12, 14, 14]);
/// ```
#[macro_export]
macro_rules! mdo(
    (
        let $p: path = $e: expr ; $( $t: tt )*
    ) => (
        { let $p = $e ; mdo! { $( $t )* } }
    );

    (
        let $p: path : $ty: ty = $e: expr ; $( $t: tt )*
    ) => (
        { let $p: $ty = $e ; mdo! { $( $t )* } }
    );

    (
        $p: pat <- $e: expr ; $( $t: tt )*
    ) => (
        bind($e, move |$p| mdo! { $( $t )* } )
    );

    (
        $p: pat : $ty: ty <- $e: expr ; $( $t: tt )*
    ) => (
        bind($e, move |$p : $ty| mdo! { $( $t )* } )
    );

    (
        ign $e: expr ; $( $t: tt )*
    ) => (
        bind($e, move |_| mdo! { $( $t )* })
    );

    (
        when $e: expr ; $( $t: tt )*
    ) => (
        bind(if $e { ret(()) } else { mzero() }, move |_| mdo! { $( $t )* })
    );

    (
        ret $f: expr
    ) => (
        $f
    )
);

pub mod option {
    //! Monadic functions for Option<T>

    /// bind for Option<T>, equivalent to `m.and_then(f)`
    pub fn bind<T, U, F: FnMut(T) -> Option<U>>(m: Option<T>, mut f: F) -> Option<U> {
        match m {
            Some(a) => f(a),
            None => None
        }
    }

    /// return for Option<T>, equivalent to `Some(x)`
    pub fn ret<T>(x: T) -> Option<T> {
        Some(x)
    }

    /// mzero for Option<T>, equivalent to `None`
    pub fn mzero<T>() -> Option<T> {
        None
    }
}

pub mod result {
    //! Monadic functions for Result<T, E>

    /// bind for Result<T, E>, equivalent to `m.and_then(f)`
    pub fn bind<T, E, U, F: FnMut(T) -> Result<U, E>>(m: Result<T, E>, mut f: F) -> Result<U, E> {
        match m {
            Ok(a) => f(a),
            Err(err) => Err(err)
        }
    }

    /// return for Result<T, E>, equivalent to `Ok(x)`
    pub fn ret<T, E>(x: T) -> Result<T, E> {
        Ok(x)
    }
}

pub mod iter {
    //! Monadic functions for Iterator<T>

    use std::option;
    use std::iter::FlatMap;

    /// bind for Iterator<T, E>, equivalent to `m.flat_map(f)`
    pub fn bind<A, T, B, U, F>(m: T, f: F) -> FlatMap<A, B, T, U, F>
            where T: Iterator<Item=A>,
                  U: Iterator<Item=B>,
                  F: FnMut(A) -> U {
        m.flat_map(f)
    }

    /// return for Iterator<T>, an iterator with one value.
    pub fn ret<T>(x: T) -> option::IntoIter<T> {
        Some(x).into_iter()
    }

    /// mzero for Iterator<T>, an empty iterator.
    pub fn mzero<T>() -> option::IntoIter<T> {
        None.into_iter()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn option_bind() {
        use super::option::{bind, ret, mzero};
        let x = ret(5i);
        assert_eq!(x, Some(5i));
        let x = bind(ret(5i), |x| ret(x + 1));
        assert_eq!(x, Some(6i));
        let x = bind(ret(5i), |x| bind(ret(x + 5), |x| ret(x * 2)));
        assert_eq!(x, Some(20));
        let x = bind(ret(5i), |x| bind(if x == 0 { ret(()) } else { mzero() },
                                       |_| ret(x * 2)));
        assert_eq!(x, None);
    }

    #[test]
    fn option_mdo() {
        use super::option::{bind, ret, mzero};
        let x = mdo! {
            ret ret(5i)
        };
        assert_eq!(x, Some(5i));
        let x = mdo! {
            x <- ret(5i);
            ret ret(x + 1)
        };
        assert_eq!(x, Some(6i));
        let x = mdo! {
            x <- ret(5i);
            x <- ret(x + 5);
            ret ret(x * 2)
        };
        assert_eq!(x, Some(20i));
        let x = mdo! {
            x <- ret(5i);
            when x == 0;
            ret ret(x * 2)
        };
        assert_eq!(x, None);
    }

    #[test]
    fn let_type() {
        let _: int = mdo! {
            let i: int = 0;
            ret i
        };
    }

    #[test]
    fn iter_bind() {
        use super::iter::{bind, ret, mzero};
        let l = bind(range(0i, 3), move |x| range(x, 3));
        assert_eq!(l.collect::<Vec<int>>(), vec![0, 1, 2, 1, 2, 2]);
        let l = bind(range(0i, 3), move |x|
                     bind(range(0i, 3), move |y| ret(x + y)));
        assert_eq!(l.collect::<Vec<int>>(), vec![0, 1, 2, 1, 2, 3, 2, 3, 4]);
        let l = bind(range(1i, 11), move |z|
                     bind(range(1, z + 1), move |y|
                          bind(range(1, y + 1), move |x|
                               bind(if x * x + y * y == z * z { ret(()) }
                                    else { mzero() },
                                    move |_|
                                    ret((x, y, z))))));
        assert_eq!(l.collect::<Vec<(int, int, int)>>(), vec![(3, 4, 5), (6, 8, 10)]);
    }

    #[test]
    fn iter_mdo() {
        use super::iter::{bind, ret, mzero};
        let l = mdo! {
            x <- range(0i, 3);
            ret range(x, 3)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![0, 1, 2, 1, 2, 2]);
        let l = mdo! {
            x <- range(0i, 3);
            y <- range(0i, 3);
            ret ret(x + y)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![0, 1, 2, 1, 2, 3, 2, 3, 4]);
        let l = mdo! {
            z <- range(1i, 11);
            y <- range(1, z);
            x <- range(1, y + 1);
            let test = x * x + y * y == z * z;
            when test;
            let res = (x, y, z);
            ret ret(res)
        }.collect::<Vec<(int, int, int)>>();
        assert_eq!(l, vec![(3, 4, 5), (6, 8, 10)]);
    }

    #[test]
    fn iter_ignore() {
        use super::iter::{bind, ret};
        let l = mdo! {
            x: int <- range(0i, 5);
            ign range(0i, 2);
            ret ret(x)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4]);
    }

    #[test]
    fn ret_trick() {
        use super::iter::bind;
        let l = mdo! {
            ret <- range(0i, 5);
            ret range(0, ret)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![0, 0, 1, 0, 1, 2, 0, 1, 2, 3]);
    }

    #[test]
    fn when_trick() {
        use super::iter::{bind, ret, mzero};
        let l = mdo! {
            when <- range(0i, 5);
            when when != 3;
            ret ret(when)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![0, 1, 2, 4]);
    }

    #[test]
    fn ign_trick() {
        use super::iter::{bind, ret};
        let l = mdo! {
            ign: int <- range(0i, 5);
            ign range(0i, 0);
            ret ret(ign)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![]);
    }

    #[test]
    fn ign_useless_type_annotation_no_macro() {
        use super::iter::{bind, ret};
        let i = bind(range(0i, 5), move |i: int| bind(range(0i, 0), move |_| ret(i)));
        assert_eq!(i.collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn ign_useless_type_annotation() {
        use super::iter::{bind, ret};
        let i = mdo! {
            i: int <- range(0i, 5);
            _ <- range(0i, 0);
            ret ret(i)
        };
        assert_eq!(i.collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn mdo_doc_example() {
        use super::iter::{bind, ret, mzero};
        let l = mdo! {
            x <- range(0i, 5); // assign x to [0, 5[
            ign range(0i, 2); // duplicate each value
            when x % 2 == 0; // filter on even values
            let y = x + 5; // create y
            ret ret(y + 5) // return y + 5
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![10, 10, 12, 12, 14, 14]);
    }
}
