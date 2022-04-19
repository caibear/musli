//! Type flags available for `musli-wire`.

use musli::{Decode, Decoder};

/// Data masked into the data type.
pub const LEN_MASK: u8 = 0b000_11111;

/// The subsequent byte is the byte to read.
pub const SEE_NEXT: u8 = 0b000_11111;

/// The structure of a type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeKind {
    /// Mark.
    Mark = 0b000_00000,
    /// A single byte.
    Byte = 0b001_00000,
    /// A fixed element where the length how many bytes it consists of.
    Fixed = 0b010_00000,
    /// The next integer is using continuation integer encoding.
    Continuation = 0b011_00000,
    /// A length-prefixed byte sequence. The length bits indicate the length of
    /// the sequence unless they are all set to 1s.
    Prefixed = 0b100_00000,
    /// A length-prefixed sequence of typed values. The length bits indicate the
    /// length of the sequence unless they are all set to 1s.
    Sequence = 0b101_00000,
    /// A length-prefixed sequence of typed pairs of values.
    PairSequence = 0b110_00000,
    /// unknown type tag.
    Unknown = 0b111_00000,
}

impl TypeKind {
    const EMPTY: u8 = TypeKind::Mark as u8;
    const BYTE: u8 = TypeKind::Byte as u8;
    const FIXED: u8 = TypeKind::Fixed as u8;
    const CONTINUATION: u8 = TypeKind::Continuation as u8;
    const PREFIXED: u8 = TypeKind::Prefixed as u8;
    const SEQUENCE: u8 = TypeKind::Sequence as u8;
    const PAIR_SEQUENCE: u8 = TypeKind::PairSequence as u8;
}

/// A decoded type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeTag {
    /// The kind of the type flag.
    pub kind: TypeKind,
    /// The length of the type flag.
    pub len: u8,
}

impl TypeTag {
    /// Construct a new type tag.
    #[inline]
    pub const fn new(kind: TypeKind, len: u8) -> Self {
        Self {
            kind,
            len: if len < LEN_MASK { len } else { LEN_MASK },
        }
    }

    /// Attempt to construct a type tag with the given length embedded.
    ///
    /// Returns a tuple where the boolean indicates if the value was embedded or
    /// not.
    #[inline]
    pub const fn with_len(kind: TypeKind, len: usize) -> (Self, bool) {
        if len < LEN_MASK as usize {
            (Self::new(kind, len as u8), true)
        } else {
            (Self::new(kind, LEN_MASK), false)
        }
    }

    /// Attempt to construct a type tag with the given length embedded.
    ///
    /// Returns a tuple where the boolean indicates if the value was embedded or
    /// not.
    #[inline]
    pub const fn with_byte(kind: TypeKind, len: u8) -> (Self, bool) {
        if len < LEN_MASK {
            (Self::new(kind, len), true)
        } else {
            (Self::new(kind, LEN_MASK), false)
        }
    }

    /// Construct from a byte.
    #[inline]
    pub const fn from_byte(b: u8) -> Self {
        let len = b & LEN_MASK;

        let kind = match b & !LEN_MASK {
            TypeKind::EMPTY => TypeKind::Mark,
            TypeKind::BYTE => TypeKind::Byte,
            TypeKind::FIXED => TypeKind::Fixed,
            TypeKind::CONTINUATION => TypeKind::Continuation,
            TypeKind::PREFIXED => TypeKind::Prefixed,
            TypeKind::SEQUENCE => TypeKind::Sequence,
            TypeKind::PAIR_SEQUENCE => TypeKind::PairSequence,
            _ => TypeKind::Unknown,
        };

        Self { kind, len }
    }

    /// Coerce type flag into a byte.
    #[inline]
    pub const fn byte(self) -> u8 {
        self.kind as u8 | self.len
    }

    /// Get the embedded length as a byte.
    #[inline]
    pub const fn len(self) -> Option<u8> {
        if self.len == LEN_MASK {
            None
        } else {
            Some(self.len)
        }
    }
}

impl<'de> Decode<'de> for TypeTag {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(Self::from_byte(decoder.decode_u8()?))
    }
}
