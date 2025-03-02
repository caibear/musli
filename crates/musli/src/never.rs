//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.
//!
//! This is a private module of musli, and is not intended for use outside of
//! the implementation attributes:
//!
//! * [`#[musli::encoder]`][crate::encoder].
//! * [`#[musli::decoder]`][crate::decoder].

use core::fmt;
use core::marker;

use crate::no_std::ToOwned;

use crate::de::{
    AsDecoder, Decoder, NumberVisitor, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder,
    SizeHint, ValueVisitor, VariantDecoder,
};
use crate::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use crate::error::Error;
use crate::Context;

/// Marker type used for the [`Never`] type.
#[doc(hidden)]
pub enum NeverMarker {}

/// An uninhabitable never type which implements all possible encoders and
/// decoders. This can be used if your [Encoder] implementation doesn't
/// implement a particular function.
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::Decoder;
///
/// struct MyDecoder(u32);
///
/// #[musli::decoder]
/// impl Decoder<'_> for MyDecoder {
///     type Error = String;
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32<'buf, C>(self, cx: &mut C) -> Result<u32, C::Error>
///     where
///         C: Context<'buf, Input = Self::Error>
///     {
///         if self.0 == 42 {
///             return Ok(self.0);
///         }
///
///         Err(cx.custom("I do not have the answer..."))
///     }
/// }
/// ```
pub struct Never<A, B: ?Sized = NeverMarker, C = NeverMarker> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<(A, C, B)>,
}

impl<'de, E> Decoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;
    type Buffer = Self;
    type Pack = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Some = Self;
    type Struct = Self;
    type Variant = Self;
    type __UseMusliDecoderAttributeMacro = ();

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<E> AsDecoder for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Never<E>
    where
        Self: 'this;

    #[inline]
    fn as_decoder<'buf, C>(&self, _: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E> PairDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type First<'this> = Self
    where
        Self: 'this;

    type Second = Self;

    #[inline]
    fn first<'buf, C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn second<'buf, C>(self, _: &mut C) -> Result<Self::Second, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn skip_second<'buf, C>(self, _: &mut C) -> Result<bool, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E> VariantDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Tag<'this> = Self
    where
        Self: 'this;

    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn variant<'buf, C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn skip_variant<'buf, C>(&mut self, _: &mut C) -> Result<bool, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E> PairsDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E> SequenceDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E> PackDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E> Encoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;
    type Pack = Self;
    type Some = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Struct = Self;
    type Variant = Self;
    type __UseMusliEncoderAttributeMacro = ();

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, 'buf, O, C> NumberVisitor<'de, 'buf, C> for Never<O, C>
where
    C: Context<'buf>,
{
    type Ok = O;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, 'buf, O, T, C> ValueVisitor<'de, 'buf, C, T> for Never<O, T, C>
where
    T: ?Sized + ToOwned,
    C: Context<'buf>,
{
    type Ok = O;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<O, E> SequenceEncoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;

    type Encoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = E>,
    {
        match self._never {}
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = E>,
    {
        match self._never {}
    }
}

impl<O, E> PairsEncoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;
    type Encoder<'this> = Self where Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E> PairEncoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;
    type First<'this> = Self
    where
        Self: 'this;
    type Second<'this> = Self where Self: 'this;

    #[inline]
    fn first<'buf, C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn second<'buf, C>(&mut self, _: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E> VariantEncoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;
    type Tag<'this> = Self
    where
        Self: 'this;
    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn variant<'buf, C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        match self._never {}
    }
}
