use crate::types::{TypeKind, TypeTag};

/// Trait that encodes common behaviors of unsigned numbers.
pub trait Typed {
    /// The type flag used.
    const TYPE_FLAG: TypeTag;
}

macro_rules! implement {
    ($ty:ty, $type_flag:expr) => {
        impl Typed for $ty {
            const TYPE_FLAG: TypeTag = $type_flag;
        }
    };
}

implement!(u16, TypeTag::new(TypeKind::Fixed, 2));
implement!(u32, TypeTag::new(TypeKind::Fixed, 4));
implement!(u64, TypeTag::new(TypeKind::Fixed, 8));
implement!(u128, TypeTag::new(TypeKind::Fixed, 16));
// TODO: this needs to be easier to determine.
implement!(usize, TypeTag::new(TypeKind::Fixed, 8));
