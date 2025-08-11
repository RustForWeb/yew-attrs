// Copied from https://github.com/yewstack/yew/blob/yew-v0.21.0/packages/yew-macro/src/stringify.rs.

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Expr, Lit, LitStr};

/// Create `AttrValue` construction calls.
///
/// This is deliberately not implemented for strings to preserve spans.
pub trait Stringify {
    /// Try to turn the value into a string literal.
    fn try_into_lit(&self) -> Option<LitStr>;

    /// Like `optimize_literals` but tags static or dynamic strings with [Value]
    fn optimize_literals_tagged(&self) -> Value
    where
        Self: ToTokens,
    {
        if let Some(lit) = self.try_into_lit() {
            Value::Static(lit.to_token_stream())
        } else {
            Value::Dynamic(self.to_token_stream())
        }
    }
}

impl<T: Stringify + ?Sized> Stringify for &T {
    fn try_into_lit(&self) -> Option<LitStr> {
        (*self).try_into_lit()
    }
}

/// A stringified value that can be either static (known at compile time) or dynamic (known only at
/// runtime)
pub enum Value {
    Static(TokenStream),
    Dynamic(TokenStream),
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Value::Static(tt) | Value::Dynamic(tt) => tt.clone(),
        });
    }
}

impl Stringify for LitStr {
    fn try_into_lit(&self) -> Option<LitStr> {
        Some(self.clone())
    }
}

impl Stringify for Lit {
    fn try_into_lit(&self) -> Option<LitStr> {
        let s = match self {
            Lit::Str(v) => return v.try_into_lit(),
            Lit::Char(v) => v.value().to_string(),
            Lit::Int(v) => v.base10_digits().to_string(),
            Lit::Float(v) => v.base10_digits().to_string(),
            Lit::Bool(_) | Lit::ByteStr(_) | Lit::Byte(_) | Lit::Verbatim(_) => return None,
            _ => unreachable!("unknown Lit"),
        };
        Some(LitStr::new(&s, self.span()))
    }
}

impl Stringify for Expr {
    fn try_into_lit(&self) -> Option<LitStr> {
        if let Expr::Lit(v) = self {
            v.lit.try_into_lit()
        } else {
            None
        }
    }
}
