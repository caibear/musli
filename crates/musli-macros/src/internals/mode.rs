//! Helper for determining the mode we're currently in.

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

use crate::internals::tokens::Tokens;
use crate::internals::Only;

#[derive(Clone, Copy)]
pub(crate) enum ModePath<'a> {
    Ident(&'a syn::Ident),
    Path(&'a syn::Path),
}

impl ModePath<'_> {
    pub(crate) fn as_path(self) -> syn::Path {
        match self {
            ModePath::Ident(ident) => syn::Path::from(ident.clone()),
            ModePath::Path(path) => path.clone(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Mode<'a> {
    pub(crate) ident: Option<&'a syn::Path>,
    pub(crate) mode_path: ModePath<'a>,
    pub(crate) tokens: &'a Tokens,
    pub(crate) only: Only,
}

impl<'a> Mode<'a> {
    /// Get the mode identifier.
    pub(crate) fn mode_ident(&self) -> ModePath<'a> {
        self.mode_path
    }

    /// Construct a typed encode call.
    pub(crate) fn encode_t_encode(&self, trace: bool) -> syn::Path {
        let moded_ident = &self.mode_path;

        let (mut encode_t, name) = if trace {
            (self.tokens.trace_encode_t.clone(), "trace_encode")
        } else {
            (self.tokens.encode_t.clone(), "encode")
        };

        if let Some(segment) = encode_t.segments.last_mut() {
            add_mode_argument(moded_ident, segment);
        }

        encode_t
            .segments
            .push(syn::PathSegment::from(syn::Ident::new(
                name,
                encode_t.span(),
            )));
        encode_t
    }

    /// Construct a typed encode call.
    pub(crate) fn decode_t_decode(&self, trace: bool) -> syn::Path {
        let moded_ident = &self.mode_path;

        let (mut decode_t, method) = if trace {
            (self.tokens.trace_decode_t.clone(), "trace_decode")
        } else {
            (self.tokens.decode_t.clone(), "decode")
        };

        if let Some(segment) = decode_t.segments.last_mut() {
            add_mode_argument(moded_ident, segment);
        }

        decode_t
            .segments
            .push(syn::PathSegment::from(syn::Ident::new(
                method,
                decode_t.span(),
            )));

        decode_t
    }
}

fn add_mode_argument(moded_ident: &ModePath, last: &mut syn::PathSegment) {
    let mut arguments = syn::AngleBracketedGenericArguments {
        colon2_token: Some(<Token![::]>::default()),
        lt_token: <Token![<]>::default(),
        args: Punctuated::default(),
        gt_token: <Token![>]>::default(),
    };

    arguments
        .args
        .push(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
            qself: None,
            path: moded_ident.as_path(),
        })));

    last.arguments = syn::PathArguments::AngleBracketed(arguments);
}
