use std::fmt;
use std::vec::Vec;

use crate::context;
use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{Signed, Unsigned};

#[test]
fn test_continuation_encoding() {
    use rand::prelude::*;

    fn rt<T>(expected: T)
    where
        T: PartialEq<T> + fmt::Debug + Unsigned,
    {
        let mut out = Vec::new();
        let mut cx = crate::context::Ignore::default();
        c::encode(&mut cx, &mut out, expected).unwrap();
        c::encode(&mut cx, &mut out, expected).unwrap();
        let mut data = out.as_slice();
        let mut cx = context::Ignore::default();
        let a: T = c::decode(&mut cx, &mut data).unwrap();
        let b: T = c::decode(&mut cx, &mut data).unwrap();
        assert!(data.is_empty());
        assert_eq!(a, expected);
        assert_eq!(b, expected);
    }

    fn encode<T>(value: T) -> Vec<u8>
    where
        T: Unsigned,
    {
        let mut out = Vec::new();
        let mut cx = crate::context::Same::default();
        c::encode(&mut cx, crate::wrap::wrap(&mut out), value).unwrap();
        out
    }

    macro_rules! test {
        ($ty:ty) => {{
            rt::<$ty>(0);
            rt::<$ty>(1);
            rt::<$ty>(42);
            rt::<$ty>(127);
            rt::<$ty>(128);
            rt::<$ty>(128 << 8);
            rt::<$ty>(<$ty>::MAX);

            let mut rng = StdRng::seed_from_u64(0xfd80fd80fd80fd80);

            for _ in 0..10000 {
                let value = rng.gen::<usize>();
                rt(value);
            }
        }};
    }

    test!(usize);
    test!(u16);
    test!(u32);
    test!(u64);
    test!(u128);

    assert_eq!(encode(1000u128), vec![232, 7]);
}

#[test]
fn test_zigzag() {
    fn rt<T>(value: T, expected: T::Unsigned)
    where
        T: fmt::Debug + Signed + PartialEq,
        T::Unsigned: Unsigned<Signed = T> + fmt::Debug + PartialEq,
    {
        assert_eq!(zig::encode(value), expected);
        assert_eq!(zig::decode(expected), value);
    }

    macro_rules! test {
        ($signed:ty, $unsigned:ty) => {
            rt::<$signed>(0, 0);
            rt::<$signed>(-1, 1);
            rt::<$signed>(1, 2);
            rt::<$signed>(-2, 3);
            rt::<$signed>(2, 4);
            rt::<$signed>(<$signed>::MAX, <$unsigned>::MAX - 1);
            rt::<$signed>(<$signed>::MIN, <$unsigned>::MAX);
        };
    }

    test!(isize, usize);
    test!(i16, u16);
    test!(i32, u32);
    test!(i64, u64);
    test!(i128, u128);
}
