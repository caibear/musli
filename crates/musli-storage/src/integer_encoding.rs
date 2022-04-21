use core::fmt::{Debug, Display};
use core::hash::Hash;

use musli::error::Error;
use musli_binary_common::encoding::{Fixed, FixedLength, Variable};
use musli_binary_common::int::continuation as c;
use musli_binary_common::int::zigzag as zig;
use musli_binary_common::int::{ByteOrder, ByteOrderIo, Signed, Unsigned};
use musli_binary_common::reader::Reader;
use musli_binary_common::writer::Writer;

mod private {
    use musli_binary_common::int::{ByteOrder, Unsigned};

    pub trait Sealed {}
    impl<B> Sealed for musli_binary_common::encoding::Fixed<B> where B: ByteOrder {}
    impl Sealed for musli_binary_common::encoding::Variable {}
    impl<L, B> Sealed for musli_binary_common::encoding::FixedLength<L, B>
    where
        L: Unsigned,
        B: ByteOrder,
    {
    }
}

/// Trait which governs how integers are encoded in a binary format.
///
/// The two common implementations of this is [Variable] and [Fixed].
pub trait IntegerEncoding:
    Clone + Copy + Debug + Eq + Hash + Ord + PartialEq + PartialOrd + private::Sealed
{
    /// Governs how unsigned integers are encoded into a [Writer].
    fn encode_unsigned<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ByteOrderIo;

    /// Governs how unsigned integers are decoded from a [Reader].
    fn decode_unsigned<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: ByteOrderIo;

    /// Governs how signed integers are encoded into a [Writer].
    fn encode_signed<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo;

    /// Governs how signed integers are decoded from a [Reader].
    fn decode_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>;
}

/// Encoding formats which ensure that variably sized types (like `usize`,
/// `isize`) are encoded in a format which is platform-neutral.
pub trait UsizeEncoding: private::Sealed {
    /// Governs how usize lengths are encoded into a [Writer].
    fn encode_usize<W>(writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer;

    /// Governs how usize lengths are decoded from a [Reader].
    fn decode_usize<'de, R>(reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>;
}

/// [IntegerEncoding] and [UsizeEncoding] implementation which encodes integers
/// using zigzag variable length encoding.
impl IntegerEncoding for Variable {
    #[inline]
    fn encode_unsigned<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Unsigned,
    {
        c::encode(writer, value)
    }

    #[inline]
    fn decode_unsigned<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Unsigned,
    {
        c::decode(reader)
    }

    #[inline]
    fn encode_signed<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
    {
        c::encode(writer, zig::encode(value))
    }

    #[inline]
    fn decode_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: Unsigned<Signed = T>,
    {
        let value: T::Unsigned = c::decode(reader)?;
        Ok(zig::decode(value))
    }
}

impl UsizeEncoding for Variable {
    #[inline]
    fn encode_usize<W>(writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer,
    {
        c::encode(writer, value)
    }

    #[inline]
    fn decode_usize<'de, R>(reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>,
    {
        c::decode(reader)
    }
}

impl<B> IntegerEncoding for Fixed<B>
where
    B: ByteOrder,
{
    #[inline]
    fn encode_unsigned<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ByteOrderIo,
    {
        value.write_bytes::<_, B>(writer)
    }

    #[inline]
    fn decode_unsigned<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: ByteOrderIo,
    {
        T::read_bytes::<_, B>(reader)
    }

    #[inline]
    fn encode_signed<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo,
    {
        value.unsigned().write_bytes::<_, B>(writer)
    }

    #[inline]
    fn decode_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>,
    {
        Ok(T::Unsigned::read_bytes::<_, B>(reader)?.signed())
    }
}

impl<L, B> UsizeEncoding for FixedLength<L, B>
where
    B: ByteOrder,
    usize: TryFrom<L>,
    L: ByteOrderIo,
    L: TryFrom<usize>,
    L::Error: 'static + Debug + Display + Send + Sync,
    <usize as TryFrom<L>>::Error: 'static + Debug + Display + Send + Sync,
{
    #[inline]
    fn encode_usize<W>(writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer,
    {
        let value: L = value.try_into().map_err(W::Error::custom)?;
        value.write_bytes::<_, B>(writer)
    }

    #[inline]
    fn decode_usize<'de, R>(reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>,
    {
        usize::try_from(L::read_bytes::<_, B>(reader)?).map_err(R::Error::custom)
    }
}
