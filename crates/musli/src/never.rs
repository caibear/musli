//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.

use core::{fmt, marker};

use crate::de::{Decoder, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder};
use crate::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder};

enum NeverMarker {}

/// An uninhabitable never type which implements all possible encoders and
/// decoders. This can be used if your [Encoder] implementation doesn't
/// implement a particular function.
///
/// ```
/// #![feature(generic_associated_types)]
///
/// use std::fmt;
///
/// use musli::de::Decoder;
/// use musli::never::Never;
///
/// struct MyDecoder;
///
/// impl<Mode> Decoder<'_, Mode> for MyDecoder {
///     type Error = String;
///     type Pack = Never<Self>;
///     type Sequence = Never<Self>;
///     type Tuple = Never<Self>;
///     type Map = Never<Self>;
///     type Some = Never<Self>;
///     type Struct = Never<Self>;
///     type TupleStruct = Never<Self>;
///     type Variant = Never<Self>;
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32(self) -> Result<u32, Self::Error> {
///         Ok(42)
///     }
/// }
/// ```
pub struct Never<T> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<T>,
}

impl<'de, Mode, T> Decoder<'de, Mode> for Never<T>
where
    T: Decoder<'de, Mode>,
{
    type Error = T::Error;
    type Pack = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Some = Self;
    type Struct = Self;
    type TupleStruct = Self;
    type Variant = Self;

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, Mode, T> PairDecoder<'de, Mode> for Never<T>
where
    T: Decoder<'de, Mode>,
{
    type Error = T::Error;

    type First<'this> = Self
    where
        Self: 'this;

    type Second = Self;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn second(self) -> Result<Self::Second, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_second(self) -> Result<bool, Self::Error> {
        match self._never {}
    }
}

impl<'de, Mode, T> PairsDecoder<'de, Mode> for Never<T>
where
    T: Decoder<'de, Mode>,
{
    type Error = T::Error;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self._never {}
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        match self._never {}
    }
}

impl<'de, Mode, T> SequenceDecoder<'de, Mode> for Never<T>
where
    T: Decoder<'de, Mode>,
{
    type Error = T::Error;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self._never {}
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        match self._never {}
    }
}

impl<'de, Mode, T> PackDecoder<'de, Mode> for Never<T>
where
    T: Decoder<'de, Mode>,
{
    type Error = T::Error;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        match self._never {}
    }
}

impl<Mode, T> Encoder<Mode> for Never<T>
where
    T: Encoder<Mode>,
{
    type Ok = T::Ok;
    type Error = T::Error;
    type Pack = Self;
    type Some = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Struct = Self;
    type TupleStruct = Self;
    type Variant = Self;

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<Mode, T> SequenceEncoder<Mode> for Never<T>
where
    T: Encoder<Mode>,
{
    type Ok = T::Ok;
    type Error = T::Error;

    type Encoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}

impl<Mode, T> PairsEncoder<Mode> for Never<T>
where
    T: Encoder<Mode>,
{
    type Ok = T::Ok;
    type Error = T::Error;
    type Encoder<'this> = Self where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        match self._never {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}

impl<Mode, T> PairEncoder<Mode> for Never<T>
where
    T: Encoder<Mode>,
{
    type Ok = T::Ok;
    type Error = T::Error;
    type First<'this> = Self
    where
        Self: 'this;
    type Second<'this> = Self where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}
