use musli::{Encode, Decode};

#[derive(Encode, Decode)]
pub struct Container<'a> {
    #[musli(with = "bytes")]
    pub data: &'a [u8],
}

mod bytes {
    use musli::{Decoder, Encoder};

    pub(crate) fn encode<Mode, E>(this: &[u8], mut encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode>,
    {
        todo!()
    }

    pub(crate) fn decode<'de, Mode, D>(mut decoder: D) -> Result<Vec<u8>, D::Error>
    where
        D: Decoder<'de, Mode>,
    {
        todo!()
    }
}

fn main() {
}
