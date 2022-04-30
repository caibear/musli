use crate::en::Encoder;
use crate::mode::DefaultMode;
pub use musli_macros::Encode;

/// Trait governing how types are encoded.
pub trait Encode<Mode = DefaultMode> {
    /// Encode the given output.
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode>;
}

impl<T, Mode> Encode<Mode> for &T
where
    T: ?Sized + Encode<Mode>,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode>,
    {
        T::encode(*self, encoder)
    }
}

impl<T, Mode> Encode<Mode> for &mut T
where
    T: ?Sized + Encode<Mode>,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode>,
    {
        T::encode(*self, encoder)
    }
}
