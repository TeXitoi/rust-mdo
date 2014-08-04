#![license = "MIT"]
#![feature(macro_rules)]

#[macro_export]
macro_rules! mdo(
    (
        let $p: path = $e: expr ; $( $t: tt )*
    ) => (
        { let $p = $e ; mdo! { $( $t )* } }
    );

    (
        bind $p: pat = $e: expr ; $( $t: tt )*
    ) => (
        bind($e, |$p| mdo! { $( $t )* } )
    );

    (
        when $e: expr ; $( $t: tt )*
    ) => (
        bind(if $e { ret(()) } else { mzero() }, |_| mdo! { $( $t )* })
    );

    (
        ign $e: expr ; $( $t: tt )*
    ) => (
        bind($e, |_| mdo! { $( $t )* })
    );

    (
        $f: expr
    ) => (
        $f
    )
)

pub mod option {
    pub fn bind<T, U>(m: Option<T>, f: |T| -> Option<U>) -> Option<U> {
        m.and_then(f)
    }
    pub fn ret<T>(x: T) -> Option<T> {
        Some(x)
    }
    pub fn mzero<T>() -> Option<T> {
        None
    }
}

pub mod result {
    pub fn bind<T, E, U>(m: Result<T, E>, f: |T| -> Result<U, E>) -> Result<U, E> {
        m.and_then(f)
    }
    pub fn ret<T, E>(x: T) -> Result<T, E> {
        Ok(x)
    }
}

pub mod iter {
    use std::vec;
    use std::option;
    pub fn bind<A, I: Iterator<A>, B, U: Iterator<B>>(
        m: I, f: |A| -> U) -> vec::MoveItems<B> {
        m.flat_map(f).collect::<Vec<B>>().move_iter()
    }
    pub fn ret<T>(x: T) -> option::Item<T> {
        Some(x).move_iter()
    }
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
            ret(5i)
        };
        assert_eq!(x, Some(5i));
        let x = mdo! {
            bind x = ret(5i);
            ret(x + 1)
        };
        assert_eq!(x, Some(6i));
        let x = mdo! {
            bind x = ret(5i);
            bind x = ret(x + 5);
            ret(x * 2)
        };
        assert_eq!(x, Some(20i));
        let x = mdo! {
            bind x = ret(5i);
            when x == 0;
            ret(x * 2)
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
            bind x = range(0i, 3);
            range(x, 3)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![0, 1, 2, 1, 2, 2]);
        let l = mdo! {
            bind x = range(0i, 3);
            bind y = range(0i, 3);
            ret(x + y)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![0, 1, 2, 1, 2, 3, 2, 3, 4]);
        let l = mdo! {
            bind z = range(1i, 11);
            bind y = range(1, z);
            bind x = range(1, y + 1);
            let test = x * x + y * y == z * z;
            when test;
            let res = (x, y, z);
            ret(res)
        }.collect::<Vec<(int, int, int)>>();
        assert_eq!(l, vec![(3, 4, 5), (6, 8, 10)]);
    }

    #[test]
    fn iter_ignore() {
        use super::iter::{bind, ret};
        let l = mdo! {
            bind x = range(0i, 5);
            ign range(0i, 2);
            ret(x)
        }.collect::<Vec<int>>();
        assert_eq!(l, vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4]);
    }
}
