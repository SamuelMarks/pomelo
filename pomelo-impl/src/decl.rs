pub use syn::{Ident, Type, ItemEnum};
pub use proc_macro2::{Group, TokenStream};
use quote;

#[derive(Debug, Copy, Clone)]
pub enum Associativity {
    Left,
    Right,
    None,
}

#[derive(Debug)]
pub enum Decl {
    Type(Ident, Type),
    Assoc(Associativity, Vec<Ident>),
    DefaultType(Type),
    ExtraArgument(Type),
    StartSymbol(Ident),
    Fallback(Ident, Vec<Ident>),
    Wildcard(Ident),
    TokenClass(Ident, Vec<Ident>),
    Token(ItemEnum),
    Rule {
        lhs: Ident,
        rhs: Vec<(Vec<Ident>, Option<Ident>)>,
        action: Option<Group>,
        prec: Option<Ident>,
    }
}

pub fn tokens_to_string(t: impl quote::ToTokens) -> String {
    let mut s = TokenStream::new();
    t.to_tokens(&mut s);
    s.to_string()
}
