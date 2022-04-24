//! Trait for governing how a particular source of bytes is read.
//!
//! `musli` requires all sources to reference the complete data being read from
//! it which allows it to make the assumption the bytes are always returned with
//! the `'de` lifetime.

use core::{fmt, slice};
use std::{marker, ops::Range, ptr};

use musli::error::Error;

/// A reader where the current position is exactly known.
pub trait PositionedReader<'de>: Reader<'de> {
    /// Target which implements [PositionedReader].
    type PositionedReaderTarget<'this>: PositionedReader<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Deref the positioned reader.
    fn deref_positioned_reader_mut(&mut self) -> Self::PositionedReaderTarget<'_>;

    /// The exact position of a reader.
    fn pos(&self) -> usize;
}

/// Trait governing how a source of bytes is read.
///
/// This requires the reader to be able to hand out contiguous references to the
/// byte source through [Reader::read_bytes].
pub trait Reader<'de> {
    /// Error type raised by the current reader.
    type Error: Error;

    /// Helper when dereffing the reader.
    type ReaderTarget<'this>: Reader<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Deref the reader to the given target.
    fn deref_reader_mut(&mut self) -> Self::ReaderTarget<'_>;

    /// Skip over the given number of bytes.
    fn skip(&mut self, n: usize) -> Result<(), Self::Error>;

    /// Read a slice into the given buffer.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let source = self.read_bytes(buf.len())?;
        buf.copy_from_slice(source);
        Ok(())
    }

    /// Read a slice out of the current reader.
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error>;

    /// Read a single byte.
    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        let [byte] = self.read_array::<1>()?;
        Ok(byte)
    }

    /// Read an array out of the current reader.
    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        let mut output = [0u8; N];
        output.copy_from_slice(self.read_bytes(N)?);
        Ok(output)
    }

    /// Keep an accurate record of the position within the reader.
    fn with_position(self) -> WithPosition<Self>
    where
        Self: Sized,
    {
        WithPosition {
            pos: 0,
            reader: self,
        }
    }

    /// Keep an accurate record of the position within the reader.
    fn limit(self, limit: usize) -> Limit<Self>
    where
        Self: Sized,
    {
        Limit {
            remaining: limit,
            reader: self,
        }
    }
}

decl_message_repr!(SliceReaderErrorRepr, "error reading from slice");

/// An error raised while decoding a slice.
#[derive(Debug)]
pub struct SliceReaderError(SliceReaderErrorRepr);

impl fmt::Display for SliceReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for SliceReaderError {
    #[inline]
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self(SliceReaderErrorRepr::collect(message))
    }

    #[inline]
    fn collect_from_display<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(SliceReaderErrorRepr::collect(message))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SliceReaderError {}

impl<'de> Reader<'de> for &'de [u8] {
    type Error = SliceReaderError;
    type ReaderTarget<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn deref_reader_mut(&mut self) -> Self::ReaderTarget<'_> {
        self
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        if self.len() < n {
            return Err(SliceReaderError::custom("buffer underflow"));
        }

        let (_, tail) = self.split_at(n);
        *self = tail;
        Ok(())
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error> {
        if self.len() < n {
            return Err(SliceReaderError::custom("buffer underflow"));
        }

        let (head, tail) = self.split_at(n);
        *self = tail;
        Ok(head)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        if self.len() < buf.len() {
            return Err(SliceReaderError::custom("buffer underflow"));
        }

        let (head, tail) = self.split_at(buf.len());
        buf.copy_from_slice(head);
        *self = tail;
        Ok(())
    }
}

/// An efficient [Reader] wrapper around a slice.
pub struct SliceReader<'de> {
    range: Range<*const u8>,
    _marker: marker::PhantomData<&'de [u8]>,
}

impl<'de> SliceReader<'de> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub fn new(slice: &'de [u8]) -> Self {
        Self {
            range: slice.as_ptr_range(),
            _marker: marker::PhantomData,
        }
    }
}

impl<'de> Reader<'de> for SliceReader<'de> {
    type Error = SliceReaderError;
    type ReaderTarget<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn deref_reader_mut(&mut self) -> Self::ReaderTarget<'_> {
        self
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.range.start = bounds_check_add(&self.range, n)?;
        Ok(())
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error> {
        let outcome = bounds_check_add(&self.range, n)?;

        unsafe {
            let bytes = slice::from_raw_parts(self.range.start, n);
            self.range.start = outcome;
            Ok(bytes)
        }
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let outcome = bounds_check_add(&self.range, buf.len())?;

        unsafe {
            ptr::copy_nonoverlapping(self.range.start, buf.as_mut_ptr(), buf.len());
            self.range.start = outcome;
        }

        Ok(())
    }
}

#[inline]
fn bounds_check_add(range: &Range<*const u8>, len: usize) -> Result<*const u8, SliceReaderError> {
    let outcome = range.start.wrapping_add(len);

    if outcome > range.end || outcome < range.start {
        Err(SliceReaderError::custom("buffer underflow"))
    } else {
        Ok(outcome)
    }
}

/// Keep a record of the current position.
///
/// Constructed through [Reader::with_position].
pub struct WithPosition<R> {
    pos: usize,
    reader: R,
}

impl<'de, R> PositionedReader<'de> for WithPosition<R>
where
    R: Reader<'de>,
{
    type PositionedReaderTarget<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn deref_positioned_reader_mut(&mut self) -> Self::PositionedReaderTarget<'_> {
        self
    }

    #[inline]
    fn pos(&self) -> usize {
        self.pos
    }
}

impl<'de, R> Reader<'de> for WithPosition<R>
where
    R: Reader<'de>,
{
    type Error = R::Error;
    type ReaderTarget<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn deref_reader_mut(&mut self) -> Self::ReaderTarget<'_> {
        self
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.reader.skip(n)?;
        self.pos += n;
        Ok(())
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error> {
        let bytes = self.reader.read_bytes(n)?;
        self.pos += bytes.len();
        Ok(bytes)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.reader.read(buf)?;
        self.pos += buf.len();
        Ok(())
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        let b = self.reader.read_byte()?;
        self.pos += 1;
        Ok(b)
    }

    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        let array = self.reader.read_array()?;
        self.pos += N;
        Ok(array)
    }
}

/// Limit the number of bytes that can be read out of a reader to the specified limit.
///
/// Constructed through [Reader::limit].
pub struct Limit<R> {
    remaining: usize,
    reader: R,
}

impl<'de, R> Limit<R>
where
    R: Reader<'de>,
{
    #[inline]
    fn bounds_check(&mut self, n: usize) -> Result<(), R::Error> {
        match self.remaining.checked_sub(n) {
            Some(remaining) => {
                self.remaining = remaining;
                Ok(())
            }
            None => Err(R::Error::custom("out of bounds")),
        }
    }
}

impl<'de, R> PositionedReader<'de> for Limit<R>
where
    R: PositionedReader<'de>,
{
    type PositionedReaderTarget<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn deref_positioned_reader_mut(&mut self) -> Self::PositionedReaderTarget<'_> {
        self
    }

    #[inline]
    fn pos(&self) -> usize {
        self.reader.pos()
    }
}

impl<'de, R> Reader<'de> for Limit<R>
where
    R: Reader<'de>,
{
    type Error = R::Error;
    type ReaderTarget<'this> = &'this mut Limit<R> where Self: 'this;

    #[inline]
    fn deref_reader_mut(&mut self) -> Self::ReaderTarget<'_> {
        self
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.bounds_check(n)?;
        self.reader.skip(n)
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error> {
        self.bounds_check(n)?;
        self.reader.read_bytes(n)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.bounds_check(buf.len())?;
        self.reader.read(buf)
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        self.bounds_check(1)?;
        self.reader.read_byte()
    }

    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        self.bounds_check(N)?;
        self.reader.read_array()
    }
}

// Forward implementations.

impl<'de, R> PositionedReader<'de> for &mut R
where
    R: ?Sized + PositionedReader<'de>,
{
    type PositionedReaderTarget<'this> = R::PositionedReaderTarget<'this> where Self: 'this;

    #[inline]
    fn deref_positioned_reader_mut(&mut self) -> Self::PositionedReaderTarget<'_> {
        (**self).deref_positioned_reader_mut()
    }

    #[inline]
    fn pos(&self) -> usize {
        (**self).pos()
    }
}

impl<'de, R> Reader<'de> for &mut R
where
    R: ?Sized + Reader<'de>,
{
    type Error = R::Error;
    type ReaderTarget<'this> = R::ReaderTarget<'this> where Self: 'this;

    #[inline]
    fn deref_reader_mut(&mut self) -> Self::ReaderTarget<'_> {
        (**self).deref_reader_mut()
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        (**self).skip(n)
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error> {
        (**self).read_bytes(n)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        (**self).read(buf)
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        (**self).read_byte()
    }

    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        (**self).read_array()
    }
}
