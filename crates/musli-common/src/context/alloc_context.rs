use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;
use core::ptr;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use musli::context::Error;
use musli::Context;

use crate::context::rich_error::{RichError, Step};

/// Buffer used in combination with [`AllocContext`].
///
/// This can be safely re-used.
#[derive(Default)]
pub struct AllocBuf {
    string: String,
}

impl AllocBuf {
    /// De-allocate the underlying buffer, this can be useful if you're dealing
    /// with infrequent decodings which uses large keys and you don't want the
    /// memory to keep being reserved.
    pub fn free(&mut self) {
        self.string = String::new();
    }
}

/// A rich context which uses allocations and tracks the exact location of every
/// error.
pub struct AllocContext<'buf, E> {
    mark: usize,
    buf: ptr::NonNull<AllocBuf>,
    errors: Vec<(Vec<Step<String>>, Range<usize>, E)>,
    path: Vec<Step<String>>,
    include_type: bool,
    _marker: PhantomData<(&'buf mut AllocBuf, E)>,
}

impl<'buf, E> AllocContext<'buf, E> {
    /// Construct a new context which uses allocations to store arbitrary
    /// amounts of diagnostics about decoding.
    ///
    /// Or at least until we run out of memory.
    pub fn new(buf: &'buf mut AllocBuf) -> Self {
        Self {
            mark: 0,
            buf: buf.into(),
            errors: Vec::new(),
            path: Vec::new(),
            include_type: false,
            _marker: PhantomData,
        }
    }

    /// Configure the context to visualize type information, and not just
    /// variant and fields.
    pub fn include_type(&mut self) -> &mut Self {
        self.include_type = true;
        self
    }

    /// Iterate over all collected errors.
    pub fn iter(&self) -> impl Iterator<Item = RichError<'_, String, E>> {
        self.errors
            .iter()
            .map(|(path, range, error)| RichError::new(path, 0, range.clone(), error))
    }
}

impl<'buf, E> Context<'buf> for AllocContext<'buf, E>
where
    E: musli::error::Error,
{
    type Input = E;
    type Error = Error;
    type Mark = usize;

    #[inline(always)]
    fn report<T>(&mut self, error: T) -> Self::Error
    where
        E: From<T>,
    {
        self.errors
            .push((self.path.clone(), self.mark..self.mark, E::from(error)));
        Error
    }

    #[inline]
    fn marked_report<T>(&mut self, mark: Self::Mark, message: T) -> Self::Error
    where
        E: From<T>,
    {
        self.errors
            .push((self.path.clone(), mark..self.mark, E::from(message)));
        Error
    }

    #[inline(always)]
    fn custom<T>(&mut self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.errors
            .push((self.path.clone(), self.mark..self.mark, E::custom(message)));
        Error
    }

    #[inline(always)]
    fn message<T>(&mut self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.errors
            .push((self.path.clone(), self.mark..self.mark, E::message(message)));
        Error
    }

    #[inline]
    fn marked_message<T>(&mut self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.errors
            .push((self.path.clone(), mark..self.mark, E::message(message)));
        Error
    }

    #[inline]
    fn mark(&mut self) -> Self::Mark {
        self.mark
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.mark = self.mark.wrapping_add(n);
    }

    #[inline(always)]
    fn store_string(&mut self, s: &str) {
        // SAFETY: we're holding onto a mutable reference to the string so it
        // must be live for the duration of the context.
        let buf = unsafe { self.buf.as_mut() };
        buf.string.clear();
        buf.string.push_str(s);
    }

    #[inline(always)]
    fn get_string<'a>(&self) -> Option<&'buf str> {
        // SAFETY: we're holding onto a mutable reference to the string so it
        // must be live for the duration of the context.
        let buf = unsafe { self.buf.as_ref() };
        Some(&buf.string)
    }

    #[inline]
    fn enter_named_field<T>(&mut self, name: &'static str, _: T)
    where
        T: fmt::Display,
    {
        self.path.push(Step::Named(name));
    }

    #[inline]
    fn enter_unnamed_field<T>(&mut self, index: u32, _: T)
    where
        T: fmt::Display,
    {
        self.path.push(Step::Unnamed(index));
    }

    #[inline]
    fn leave_field(&mut self) {
        self.path.pop();
    }

    #[inline]
    fn enter_struct(&mut self, name: &'static str) {
        if self.include_type {
            self.path.push(Step::Struct(name));
        }
    }

    #[inline]
    fn leave_struct(&mut self) {
        if self.include_type {
            self.path.pop();
        }
    }

    #[inline]
    fn enter_enum(&mut self, name: &'static str) {
        if self.include_type {
            self.path.push(Step::Enum(name));
        }
    }

    #[inline]
    fn leave_enum(&mut self) {
        if self.include_type {
            self.path.pop();
        }
    }

    #[inline]
    fn enter_variant<T>(&mut self, name: &'static str, _: T) {
        self.path.push(Step::Variant(name));
    }

    #[inline]
    fn leave_variant(&mut self) {
        self.path.pop();
    }

    #[inline]
    fn enter_sequence_index(&mut self, index: usize) {
        self.path.push(Step::Index(index));
    }

    #[inline]
    fn leave_sequence_index(&mut self) {
        self.path.pop();
    }

    #[inline]
    fn enter_map_key<T>(&mut self, field: T)
    where
        T: fmt::Display,
    {
        self.path.push(Step::Key(field.to_string()));
    }

    #[inline]
    fn leave_map_key(&mut self) {
        self.path.pop();
    }
}
