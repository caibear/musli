#![allow(unused)]

use std::collections::HashMap;

use musli::{Decode, Encode};
use musli_common::context::{NoStdBuf, NoStdContext};

#[derive(Encode)]
struct From {
    values: HashMap<String, String>,
}

#[derive(Decode)]
struct Collection {
    #[musli(trace)]
    values: HashMap<String, u32>,
}

#[test]
fn trace_no_std() {
    let mut buf = NoStdBuf::default();

    let mut cx = NoStdContext::new(&mut buf);

    let mut values = HashMap::new();

    values.insert("Hello".to_string(), "World".to_string());

    let from = From { values };

    let encoding = musli_json::Encoding::new();

    let Ok(bytes) = encoding.to_vec_with(&mut cx, &from) else {
        for error in cx.iter() {
            panic!("{error}");
        }

        unreachable!()
    };

    let mut cx = NoStdContext::new(&mut buf);

    let Ok(..) = encoding.from_slice_with::<_, Collection>(&mut cx, &bytes) else {
        if let Some(error) = cx.iter().next() {
            assert_eq!(error.to_string(), ".values[Hello]: invalid numeric (at bytes 15-16)");
            return;
        }

        unreachable!()
    };

    panic!("expected decoding to error");
}
