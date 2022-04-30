//! Module that defines [StorageEncoding] whith allows for customization of the
//! encoding format, and the [DEFAULT] encoding configuration.

use core::marker;
#[cfg(feature = "std")]
use std::io;

use crate::de::StorageDecoder;
use crate::en::StorageEncoder;
use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use musli::{Decode, DefaultMode, Encode};
use musli_common::encoding::{Fixed, FixedLength, Variable};
use musli_common::fixed_bytes::{FixedBytes, FixedBytesWriterError};
use musli_common::int::{BigEndian, LittleEndian, NetworkEndian};
use musli_common::reader::{Reader, SliceReader, SliceReaderError};
#[cfg(feature = "std")]
use musli_common::writer::VecWriterError;
use musli_common::writer::Writer;

/// The default configuration.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [zigzag] with [continuation] encoding for
/// numbers.
///
/// [zigzag]: musli_common::int::zigzag
/// [continuation]: musli_common::int::continuation
pub const DEFAULT: StorageEncoding<Variable, Variable, DefaultMode> = StorageEncoding::new();

/// Encode the given value to the given [Writer] using the [DEFAULT]
/// configuration.
#[inline]
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), W::Error>
where
    W: Writer,
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.encode(writer, value)
}

/// Encode the given value to the given [Write][io::Write] using the [DEFAULT]
/// configuration.
#[cfg(feature = "std")]
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), io::Error>
where
    W: io::Write,
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_writer(writer, value)
}

/// Encode the given value to a [Vec] using the [DEFAULT] configuration.
#[cfg(feature = "std")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, VecWriterError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a fixed-size bytes using the [DEFAULT]
/// configuration.
#[inline]
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, FixedBytesWriterError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_fixed_bytes::<N, _>(value)
}

/// Decode the given type `T` from the given [Reader] using the [DEFAULT]
/// configuration.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, R::Error>
where
    R: Reader<'de>,
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.decode(reader)
}

/// Decode the given type `T` from the given slice using the [DEFAULT]
/// configuration.
#[inline]
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, SliceReaderError>
where
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.from_slice(bytes)
}

/// Setting up encoding with parameters.
#[derive(Clone, Copy)]
pub struct StorageEncoding<I, L, Mode = DefaultMode>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    _marker: marker::PhantomData<(I, L, Mode)>,
}

impl StorageEncoding<Variable, Variable, DefaultMode> {
    /// Construct a new [StorageEncoding] instance which uses [Variable] integer
    /// encoding.
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```rust
    /// use musli_storage::{StorageEncoding, Fixed, Variable};
    /// use musli::{Encode, Decode};
    ///
    /// const CONFIG: StorageEncoding<Fixed, Variable> = StorageEncoding::new()
    ///     .with_fixed_integers();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// struct Struct<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut out = Vec::new();
    ///
    /// let expected = Struct {
    ///     name: "Aristotle",
    ///     age: 61,
    /// };
    ///
    /// CONFIG.encode(&mut out, &expected)?;
    /// let actual = CONFIG.decode(&out[..])?;
    ///
    /// assert_eq!(expected, actual);
    /// # Ok(()) }
    /// ```
    pub const fn new() -> Self {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }
}

impl<I, L, Mode> StorageEncoding<I, L, Mode>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    /// Modify the encoding mode.
    pub const fn with_mode<M>(self) -> StorageEncoding<I, L, M> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable integer encoding.
    pub const fn with_variable_integers(self) -> StorageEncoding<Variable, L, Mode> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer encoding.
    pub const fn with_fixed_integers(self) -> StorageEncoding<Fixed, L, Mode> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer little-endian encoding.
    pub const fn with_fixed_integers_le(self) -> StorageEncoding<Fixed<LittleEndian>, L, Mode> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer big-endian encoding.
    pub const fn with_fixed_integers_be(self) -> StorageEncoding<Fixed<BigEndian>, L, Mode> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer network-endian encoding
    /// (Default).
    pub const fn with_fixed_integers_ne(self) -> StorageEncoding<Fixed<NetworkEndian>, L, Mode> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable length encoding.
    pub const fn with_variable_lengths(self) -> StorageEncoding<I, Variable, Mode> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 32-bit encoding when encoding
    /// lengths.
    pub const fn with_fixed_lengths(self) -> StorageEncoding<I, FixedLength<u32>, Mode> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 64-bit encoding when encoding
    /// lengths.
    pub const fn with_fixed_lengths64(self) -> StorageEncoding<I, FixedLength<u64>, Mode> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Encode the given value to the given [Writer] using the current
    /// configuration.
    #[inline]
    pub fn encode<W, T>(self, writer: W, value: &T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ?Sized + Encode<Mode>,
    {
        T::encode(value, StorageEncoder::<Mode, _, I, L>::new(writer))
    }

    /// Encode the given value to the given [Write][io::Write] using the current
    /// configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), io::Error>
    where
        W: io::Write,
        T: ?Sized + Encode<Mode>,
    {
        let writer = musli_common::io::wrap(write);
        T::encode(value, StorageEncoder::<Mode, _, I, L>::new(writer))
    }

    /// Encode the given value to a [Vec] using the current configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, VecWriterError>
    where
        T: ?Sized + Encode<Mode>,
    {
        let mut data = Vec::new();
        T::encode(value, StorageEncoder::<Mode, _, I, L>::new(&mut data))?;
        Ok(data)
    }

    /// Encode the given value to a fixed-size bytes using the current
    /// configuration.
    #[inline]
    pub fn to_fixed_bytes<const N: usize, T>(
        self,
        value: &T,
    ) -> Result<FixedBytes<N>, FixedBytesWriterError>
    where
        T: ?Sized + Encode<Mode>,
    {
        let mut bytes = FixedBytes::new();
        T::encode(value, StorageEncoder::<Mode, _, I, L>::new(&mut bytes))?;
        Ok(bytes)
    }

    /// Decode the given type `T` from the given [Reader] using the current
    /// configuration.
    #[inline]
    pub fn decode<'de, R, T>(self, reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Decode<'de, Mode>,
    {
        let reader = reader.with_position();
        T::decode(StorageDecoder::<Mode, _, I, L>::new(reader))
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    #[inline]
    pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, SliceReaderError>
    where
        T: Decode<'de, Mode>,
    {
        let reader = SliceReader::new(bytes).with_position();
        T::decode(StorageDecoder::<Mode, _, I, L>::new(reader))
    }
}
