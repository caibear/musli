use musli::{Decode, Encode};
use musli_wire::{
    test::Typed,
    types::{TypeKind, TypeTag},
};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct StructWithNumbers {
    a: i16,
    b: i32,
    c: i64,
    d: i128,
}

#[test]
fn test_signed_unpacked() {
    let out = musli_wire::test::rt(StructWithNumbers {
        a: -1,
        b: 1,
        c: -3,
        d: 3,
    });

    let out = musli_wire::to_vec(&out).expect("failed to encode");

    let unpacked: Unpacked = musli_storage::decode(&out[..]).expect("failed to decode");

    assert_eq! {
        unpacked,
        Unpacked {
            count: Typed::new(TypeTag::new(TypeKind::PairSequence, 0), 4),
            a_tag: Typed::new(TypeTag::new(TypeKind::Continuation, 0), 0),
            a: Typed::new(TypeTag::new(TypeKind::Continuation, 0), 1),
            b_tag: Typed::new(TypeTag::new(TypeKind::Continuation, 0), 1),
            b: Typed::new(TypeTag::new(TypeKind::Continuation, 0), 2),
            c_tag: Typed::new(TypeTag::new(TypeKind::Continuation, 0), 2),
            c: Typed::new(TypeTag::new(TypeKind::Continuation, 0), 5),
            d_tag: Typed::new(TypeTag::new(TypeKind::Continuation, 0), 3),
            d: Typed::new(TypeTag::new(TypeKind::Continuation, 0), 6),
        }
    };

    #[derive(Debug, Clone, PartialEq, Decode)]
    #[musli(packed)]
    struct Unpacked {
        count: Typed<u8>,
        a_tag: Typed<u8>,
        a: Typed<u8>,
        b_tag: Typed<u8>,
        b: Typed<u8>,
        c_tag: Typed<u8>,
        c: Typed<u8>,
        d_tag: Typed<u8>,
        d: Typed<u8>,
    }
}
