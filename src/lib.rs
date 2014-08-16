#![license = "MIT"]
#![deny(missing_doc)]
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
        $p: pat <- $e: expr ; $( $t: tt )*
    ) => (
        bind($e, |$p| mdo! { $( $t )* } )
    );

    (
        ign $e: expr ; $( $t: tt )*
    ) => (
        bind($e, |_| mdo! { $( $t )* })
    );

    (
        when $e: expr ; $( $t: tt )*
    ) => (
        bind(if $e { ret(()) } else { mzero() }, |_| mdo! { $( $t )* })
    );

    (
        ret $f: expr
    ) => (
        $f
    )
)

pub mod option {
    //! Monadic functions for Option<T>

    /// bind for Option<T>, equivalent to `m.and_then(f)`
    pub fn bind<T, U>(m: Option<T>, f: |T| -> Option<U>) -> Option<U> {
        m.and_then(f)
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
    pub fn bind<T, E, U>(m: Result<T, E>, f: |T| -> Result<U, E>) -> Result<U, E> {
        m.and_then(f)
    }

    /// return for Result<T, E>, equivalent to `Ok(x)`
    pub fn ret<T, E>(x: T) -> Result<T, E> {
        Ok(x)
    }
}

pub mod iter {
    //! Monadic functions for Iterator<T>

    use std::vec;
    use std::option;

    /// bind for Result<T, E>, equivalent to `m.flat_map(f)`
    ///
    /// Note that the current implementation collect the result in a
    /// Vec<B> because flat_map depend on the lifetime of `f`.  It
    /// mut be fixed in the futur using a unboxed closure moved
    /// inside a flat_map like iterator.
    pub fn bind<A, I: Iterator<A>, B, U: Iterator<B>>(
        m: I, f: |A| -> U) -> vec::MoveItems<B> {
        m.flat_map(f).collect::<Vec<B>>().move_iter()
    }

    /// return for Iterator<T>, an iterator with one value.
    pub fn ret<T>(x: T) -> option::Item<T> {
        Some(x).move_iter()
    }

    /// mzero for Iterator<T>, an empty iterator.
    pub fn mzero<T>() -> option::Item<T> {
        None.move_iter()
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
    fn iter_bind() {
        use super::iter::{bind, ret, mzero};
        let mut l = bind(range(0i, 3), |x| range(x, 3));
        assert_eq!(l.collect::<Vec<int>>(), vec![0, 1, 2, 1, 2, 2]);
        let mut l = bind(range(0i, 3), |x| bind(range(0i, 3), |y| ret(x + y)));
        assert_eq!(l.collect::<Vec<int>>(), vec![0, 1, 2, 1, 2, 3, 2, 3, 4]);
        let mut l = bind(range(1i, 11),
                         |z| bind(range(1, z + 1),
                                  |y| bind(range(1, y + 1),
                                           |x| bind(if x * x + y * y == z * z { ret(()) }
                                                    else { mzero() },
                                                    |_| ret((x, y, z))))));
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
            x <- range(0i, 5);
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
            ign <- range(0i, 5);
            ign range(0i, 0);
            ret ret(ign)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![]);
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
