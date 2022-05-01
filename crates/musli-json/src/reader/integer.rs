use core::fmt;

pub(crate) use self::traits::{Signed, Unsigned};
use crate::reader::{ParseError, ParseErrorKind, Parser};

/// Error when computing integer.
#[derive(Debug)]
pub(crate) enum Error {
    /// Arithmetic overflow.
    Overflow,
    /// Decimal number encountered.
    Decimal,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Overflow => write!(f, "arithmetic overflow"),
            Error::Decimal => write!(f, "decimal number"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Fully deconstructed parts of a signed number.
pub(crate) struct SignedParts<T>
where
    T: Signed,
{
    /// Indicates if the number is negative.
    is_negative: bool,
    /// Parts of the unsigned component of the number.
    parts: Parts<T::Unsigned>,
}

impl<T> SignedParts<T>
where
    T: Signed,
{
    fn compute(self) -> Result<T, Error> {
        let Self { is_negative, parts } = self;

        let value = parts.compute()?;

        match if is_negative {
            value.negate()
        } else {
            value.signed()
        } {
            Some(value) => Ok(value),
            None => Err(Error::Overflow),
        }
    }
}

#[derive(Default)]
pub(crate) struct Exponent {
    is_negative: bool,
    value: u32,
}

/// The mantissa, or anything after a decimal point.
pub(crate) struct Mantissa<T> {
    /// The value of the mantissa.
    value: T,
    /// The exponent of the mantissa.
    exp: u32,
}

impl<T> Default for Mantissa<T>
where
    T: Unsigned,
{
    fn default() -> Self {
        Self {
            value: T::ZERO,
            exp: 0u32,
        }
    }
}

/// Fully deconstructed parts of an unsigned number.
pub(crate) struct Parts<T> {
    /// The base, or everything before a decimal point.
    base: T,
    /// The mantissa, or anything after a decimal point.
    m: Mantissa<T>,
    /// The exponent of the number.
    e: Exponent,
}

impl<T> Parts<T>
where
    T: Unsigned,
{
    fn compute(self) -> Result<T, Error> {
        macro_rules! check {
            ($expr:expr, $kind:ident) => {
                match $expr {
                    Some(value) => value,
                    None => return Err(Error::$kind),
                }
            };
        }

        let Self { mut base, m, e } = self;

        if e.value == 0 {
            if !m.value.is_zero() {
                return Err(Error::Decimal);
            }

            return Ok(base);
        }

        if !e.is_negative {
            // Decoding the specified mantissa would result in a fractional number.
            let mantissa_exp = check!(e.value.checked_sub(m.exp), Decimal);

            if !base.is_zero() {
                base = check!(base.checked_pow10(e.value), Overflow);
            }

            let base = check! {
                m.value
                    .checked_pow10(mantissa_exp)
                    .and_then(|m| base.checked_add(m)),
                Overflow
            };

            return Ok(base);
        }

        if !m.value.is_zero() {
            return Err(Error::Decimal);
        }

        for _ in 0..e.value {
            base = check!(base.div_mod_ten(), Decimal);
        }

        Ok(base)
    }
}

/// Implementation to skip over a well-formed JSON number.
pub(crate) fn skip_number<'de, P>(p: &mut P) -> Result<(), ParseError>
where
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();

    if p.peek_byte()? == Some(b'-') {
        p.skip(1)?;
    }

    match p.read_byte()? {
        b'0' => (),
        b if is_digit_nonzero(b) => {
            p.consume_while(is_digit_nonzero)?;
        }
        _ => {
            return Err(ParseError::spanned(
                start,
                p.pos(),
                ParseErrorKind::InvalidNumeric,
            ));
        }
    }

    if p.peek_byte()? == Some(b'.') {
        p.skip(1)?;
        p.consume_while(is_digit)?;
    }

    if matches!(p.peek_byte()?, Some(b'e') | Some(b'E')) {
        p.skip(1)?;

        match p.peek_byte()? {
            Some(b'-') => {
                p.skip(1)?;
            }
            Some(b'+') => {
                p.skip(1)?;
            }
            _ => (),
        };

        p.consume_while(is_digit)?;
    }

    Ok(())
}

/// Fully parse an unsigned value.
pub(crate) fn parse_unsigned<'de, T, P>(p: &mut P) -> Result<T, ParseError>
where
    T: Unsigned,
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();

    match decode_unsigned(p)?.compute() {
        Ok(value) => Ok(value),
        Err(error) => Err(ParseError::spanned(
            start,
            p.pos(),
            ParseErrorKind::IntegerError(error),
        )),
    }
}

pub(crate) fn decode_unsigned<'de, T, P>(p: &mut P) -> Result<Parts<T>, ParseError>
where
    T: Unsigned,
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();
    decode_unsigned_inner(p, start)
}

/// Decode a signed integer.
pub(crate) fn decode_signed<'de, T, P>(p: &mut P) -> Result<SignedParts<T>, ParseError>
where
    T: Signed,
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();

    let is_negative = if p.peek_byte()? == Some(b'-') {
        p.skip(1)?;
        true
    } else {
        false
    };

    let parts = decode_unsigned_inner::<T::Unsigned, _>(p, start)?;
    Ok(SignedParts { is_negative, parts })
}

/// Fully parse a signed value.
pub(crate) fn parse_signed<'de, T, P>(p: &mut P) -> Result<T, ParseError>
where
    T: Signed,
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();

    match decode_signed(p)?.compute() {
        Ok(value) => Ok(value),
        Err(error) => Err(ParseError::spanned(
            start,
            p.pos(),
            ParseErrorKind::IntegerError(error),
        )),
    }
}

/// Generically decode a single (whole) integer from a stream of bytes abiding
/// by JSON convention for format.
fn decode_unsigned_inner<'de, T, P>(p: &mut P, start: u32) -> Result<Parts<T>, ParseError>
where
    T: Unsigned,
    P: ?Sized + Parser<'de>,
{
    let base = match p.read_byte()? {
        b'0' => T::ZERO,
        b if is_digit_nonzero(b) => {
            let mut base = T::from_byte(b - b'0');

            while let Some(true) = p.peek_byte()?.map(is_digit) {
                base = digit(base, p, start)?;
            }

            base
        }
        _ => {
            return Err(ParseError::spanned(
                start,
                p.pos(),
                ParseErrorKind::InvalidNumeric,
            ));
        }
    };

    let mut m = Mantissa::<T>::default();

    if let Some(b'.') = p.peek_byte()? {
        p.skip(1)?;

        // NB: we use unchecked operations over mantissa_exp since the mantissa
        // for any supported type would overflow long before this.
        m.exp += decode_zeros(p)?;

        // Stored zeros so that the last segment of zeros can be ignored since
        // they have no bearing on the value of the integer.
        let mut zeros = 0;

        while let Some(true) = p.peek_byte()?.map(is_digit) {
            // Accrue accumulated zeros.
            if zeros > 0 {
                m.exp += zeros;
                m.value = match m.value.checked_pow10(zeros) {
                    Some(mantissa) => mantissa,
                    None => {
                        return Err(ParseError::spanned(
                            start,
                            p.pos(),
                            ParseErrorKind::IntegerError(Error::Overflow),
                        ))
                    }
                };
            }

            m.exp += 1;
            m.value = digit(m.value, p, start)?;
            zeros = decode_zeros(p)?;
        }
    }

    let e = if matches!(p.peek_byte()?, Some(b'e' | b'E')) {
        p.skip(1)?;
        decode_exponent(p, start)?
    } else {
        Exponent::default()
    };

    Ok(Parts { base, m, e })
}

/// Decode an exponent.
fn decode_exponent<'de, P>(p: &mut P, start: u32) -> Result<Exponent, ParseError>
where
    P: ?Sized + Parser<'de>,
{
    let mut e = Exponent::default();

    match p.peek_byte()? {
        Some(b'-') => {
            p.skip(1)?;
            e.is_negative = true
        }
        Some(b'+') => {
            p.skip(1)?;
        }
        _ => (),
    };

    while let Some(true) = p.peek_byte()?.map(is_digit) {
        e.value = digit(e.value, p, start)?;
    }

    Ok(e)
}

/// Decode a single digit into `out`.
#[inline]
fn digit<'de, T, P>(mut out: T, p: &mut P, start: u32) -> Result<T, ParseError>
where
    T: Unsigned,
    P: ?Sized + Parser<'de>,
{
    out = match out.checked_mul10() {
        Some(value) => value,
        None => {
            return Err(ParseError::spanned(
                start,
                p.pos(),
                ParseErrorKind::IntegerError(Error::Overflow),
            ));
        }
    };

    Ok(out + T::from_byte(p.read_byte()? - b'0'))
}

/// Decode sequence of zeros.
fn decode_zeros<'de, P>(p: &mut P) -> Result<u32, ParseError>
where
    P: ?Sized + Parser<'de>,
{
    let mut count = 0;

    while let Some(b'0') = p.peek_byte()? {
        count += 1;
        p.skip(1)?;
    }

    Ok(count)
}

// Test if b is numeric.
#[inline]
fn is_digit(b: u8) -> bool {
    b'0' <= b && b <= b'9'
}

// Test if b is numeric.
#[inline]
fn is_digit_nonzero(b: u8) -> bool {
    b'1' <= b && b <= b'9'
}

mod traits {
    use core::fmt;
    use core::ops::{Add, Not};

    pub(crate) trait Unsigned: Sized + fmt::Debug + Add<Self, Output = Self> {
        type Signed: Signed<Unsigned = Self>;

        const ZERO: Self;

        fn from_byte(b: u8) -> Self;

        fn is_zero(&self) -> bool;

        fn checked_pow10(self, exp: u32) -> Option<Self>;

        fn checked_mul10(self) -> Option<Self>;

        fn checked_add(self, other: Self) -> Option<Self>;

        fn checked_mul(self, other: Self) -> Option<Self>;

        fn div_mod_ten(self) -> Option<Self>;

        fn checked_pow(self, exp: u32) -> Option<Self>;

        fn negate(self) -> Option<Self::Signed>;

        fn signed(self) -> Option<Self::Signed>;
    }

    pub(crate) trait Signed: Sized + fmt::Debug {
        type Unsigned: Unsigned<Signed = Self>;

        fn negate(self) -> Option<Self::Unsigned>;
    }

    macro_rules! count {
        (()) => { 0 };
        ((_)) => { 1 };
        ((_ _)) => { 2 };
        ((_ _ _)) => { 3 };
        ((_ _ _ _)) => { 4 };
        ((_ _ _ _ _)) => { 5 };
        ((_ _ _ _ _ _)) => { 6 };
        ((_ _ _ _ _ _ _)) => { 7 };
        ((_ _ _ _ _ _ _ _)) => { 8 };
        ((_ _ _ _ _ _ _ _ _)) => { 9 };
        ((_ _ _ _ _ _ _ _ _ _)) => { 10 };
        ((_ _ _ _ _ _ _ _ _ _ _)) => { 11 };
        ((_ _ _ _ _ _ _ _ _ _ _ _)) => { 12 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _)) => { 13 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 14 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 15 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 16 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 17 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 18 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 19 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 20 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 21 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 22 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 23 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 24 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 25 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 26 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 27 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 28 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 29 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 30 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 31 };
        ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 32 };

        (($($s:tt)*) $first:tt $($tt:tt)*) => {
            count!(($($s)* _) $($tt)*)
        };
    }

    macro_rules! unsigned {
        ($unsigned:ty, $signed:ty, [$($pows:literal),* $(,)?]) => {
            impl Unsigned for $unsigned {
                type Signed = $signed;

                const ZERO: Self = 0;

                #[inline]
                fn from_byte(b: u8) -> Self {
                    b as $unsigned
                }

                #[inline]
                fn is_zero(&self) -> bool {
                    *self == 0
                }

                #[inline]
                fn checked_pow10(self, exp: u32) -> Option<Self> {
                    static POWS: [$unsigned; count!(() $($pows)*)] = [
                        $($pows),*
                    ];

                    if let Some(exp) = POWS.get(exp as usize) {
                        self.checked_mul(*exp)
                    } else {
                        self.checked_mul(10.checked_pow(exp)?)
                    }
                }

                #[inline]
                fn checked_mul10(self) -> Option<Self> {
                    self.checked_mul(10)
                }

                #[inline]
                fn checked_add(self, other: Self) -> Option<Self> {
                    <$unsigned>::checked_add(self, other)
                }

                #[inline]
                fn checked_mul(self, other: Self) -> Option<Self> {
                    <$unsigned>::checked_mul(self, other)
                }

                #[inline]
                fn div_mod_ten(self) -> Option<Self> {
                    if self % 10 == 0 {
                        Some(self / 10)
                    } else {
                        None
                    }
                }

                #[inline]
                fn checked_pow(self, exp: u32) -> Option<Self> {
                    <$unsigned>::checked_pow(self, exp)
                }

                #[inline]
                fn negate(self) -> Option<Self::Signed> {
                    if self > (<$unsigned>::MAX >> 1) + 1 {
                        None
                    } else {
                        Some(self.not().wrapping_add(1) as $signed)
                    }
                }

                #[inline]
                fn signed(self) -> Option<Self::Signed> {
                    if self > <$unsigned>::MAX >> 1 {
                        None
                    } else {
                        Some(self as $signed)
                    }
                }
            }

            impl Signed for $signed {
                type Unsigned = $unsigned;

                #[inline]
                fn negate(self) -> Option<Self::Unsigned> {
                    if self < 0 {
                        None
                    } else {
                        Some(-self as $unsigned)
                    }
                }
            }
        };
    }

    unsigned!(u8, i8, [1, 10, 100,]);

    unsigned!(u16, i16, [1, 10, 100, 1000, 10000,]);

    unsigned!(
        u32,
        i32,
        [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,]
    );

    unsigned!(
        u64,
        i64,
        [
            1,
            10,
            100,
            1000,
            10000,
            100000,
            1000000,
            10000000,
            100000000,
            1000000000,
            10000000000,
            100000000000,
            1000000000000,
            10000000000000,
            100000000000000,
            1000000000000000,
            10000000000000000,
            100000000000000000,
            1000000000000000000,
            10000000000000000000,
        ]
    );

    unsigned!(
        u128,
        i128,
        [
            1,
            10,
            100,
            1000,
            10000,
            100000,
            1000000,
            10000000,
            100000000,
            1000000000,
            10000000000,
            100000000000,
            1000000000000,
            10000000000000,
            100000000000000,
            1000000000000000,
            10000000000000000,
            100000000000000000,
            1000000000000000000,
            10000000000000000000,
            100000000000000000000,
            1000000000000000000000,
            10000000000000000000000,
            100000000000000000000000,
            1000000000000000000000000,
            10000000000000000000000000,
            100000000000000000000000000,
            1000000000000000000000000000,
            10000000000000000000000000000,
            100000000000000000000000000000,
            1000000000000000000000000000000,
            10000000000000000000000000000000,
        ]
    );

    #[cfg(target_pointer_width = "32")]
    unsigned!(
        usize,
        isize,
        [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,]
    );

    #[cfg(target_pointer_width = "64")]
    unsigned!(
        usize,
        isize,
        [
            1,
            10,
            100,
            1000,
            10000,
            100000,
            1000000,
            10000000,
            100000000,
            1000000000,
            10000000000,
            100000000000,
            1000000000000,
            10000000000000,
            100000000000000,
            1000000000000000,
            10000000000000000,
            100000000000000000,
            1000000000000000000,
            10000000000000000000,
        ]
    );
}
