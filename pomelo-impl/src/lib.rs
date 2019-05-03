#![recursion_limit="256"]
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

mod decl;
mod parser;

use decl::*;

use quote::ToTokens;

use proc_macro2::TokenTree;
use syn::{Ident, Type};

use syn::buffer::TokenBuffer;

/// From procedural_masquerade
#[doc(hidden)]
fn _extract_input(derive_input: &str) -> &str {
    let mut input = derive_input;

    for expected in &["#[allow(unused)]", "enum", "ProceduralMasqueradeDummyType", "{", "Input", "=", "(0,", "stringify!", "("] {
        input = input.trim_start();
        assert!(input.starts_with(expected), "expected prefix {:?} not found in {:?}", expected, derive_input);
        input = &input[expected.len()..];
    }

    for expected in [")", ").0,", "}"].iter().rev() {
        input = input.trim_end();
        assert!(input.ends_with(expected), "expected suffix {:?} not found in {:?}", expected, derive_input);
        let end = input.len() - expected.len();
        input = &input[..end];
    }

    input
}



#[proc_macro_derive(__pomelo_impl)]
pub fn __pomelo_structs_and_impls(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let s = input.to_string();
    let input = _extract_input(&s);

    let input : TokenTree = syn::parse_str(input).expect("Syntax error in pomelo! body");

    let buffer = TokenBuffer::new2(input.into_token_stream());
    let cursor = buffer.begin();
    let (decls, _cursor) = parse_pomelo(cursor).unwrap();

    let mut lemon = parser::Lemon::new_from_decls(decls).unwrap();
    let expanded = lemon.build().unwrap();

    let x = quote!{
        mod parser {
            #expanded
        }
    };
    x.into()
}

named!{parse_declaration -> Decl,
    alt!(
        do_parse!(
            punct!(%) >> keyword!(type) >> ident: syn!(Ident) >> typ: syn!(Type) >> option!(punct!(;)) >>
            (Decl::Type(ident, typ))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(include) >>
            code: map!(braces!(many0!(syn!(Item))), |(_,c)| c) >>
            option!(punct!(;)) >>
            (Decl::Include(code))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(left) >> toks: many0!(syn!(Ident)) >> option!(punct!(;)) >>
            (Decl::Assoc(Associativity::Left, toks))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(right) >> toks: many0!(syn!(Ident)) >> option!(punct!(;)) >>
            (Decl::Assoc(Associativity::Right, toks))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(nonassoc) >> toks: many0!(syn!(Ident)) >> option!(punct!(;)) >>
            (Decl::Assoc(Associativity::None, toks))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(default_type) >> typ: syn!(Type) >> option!(punct!(;)) >>
            (Decl::DefaultType(typ))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(extra_argument) >> typ: syn!(Type) >> option!(punct!(;)) >>
            (Decl::ExtraArgument(typ))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(start_symbol) >> id: syn!(Ident) >> option!(punct!(;)) >>
            (Decl::StartSymbol(id))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(fallback) >> fallback: syn!(Ident) >> ids: many0!(syn!(Ident)) >> option!(punct!(;)) >>
            (Decl::Fallback(fallback, ids))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(wildcard) >> id: syn!(Ident) >> option!(punct!(;)) >>
            (Decl::Wildcard(id))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(token_class) >> tk: syn!(Ident) >> ids: many0!(syn!(Ident)) >> option!(punct!(;)) >>
            (Decl::TokenClass(tk, ids))
        )
        |
        do_parse!(
            punct!(%) >> custom_keyword!(token) >> e: syn!(ItemEnum) >> option!(punct!(;)) >>
            (Decl::Token(e))
        )
        |
        parse_rule
    )
}

named!{parse_rule -> Decl,
    do_parse!(
        lhs: syn!(Ident) >>
        punct!(::) >> punct!(=) >>
        rhs: many0!(parse_rhs) >>
        action: alt!(
            do_parse!(
                //punct!(=>) >>
                action: syn!(Group) >>
                (Some(action))
            )
            |
            epsilon!() => { |_| None }
        ) >>
        prec: option!(parse_precedence_in_rule) >>
        option!(punct!(;)) >>
        (Decl::Rule { lhs, rhs, action, prec })
    )
}

named!{parse_rhs -> (Vec<Ident>, Option<Ident>),
    do_parse!(
        toks: call!(syn::punctuated::Punctuated::<Ident, Token!(|)>::parse_separated_nonempty) >>
        alias: option!(parse_alias) >>
        ((toks.into_pairs().map(|x| x.into_value()).collect(), alias))
    )
}

named!{parse_alias -> Ident,
    map!(parens!(syn!(Ident)), |(_,r)| r)
}

named!{parse_precedence_in_rule -> Ident,
    map!(brackets!(syn!(Ident)), |(_,r)| r)
}

named!{parse_pomelo -> Vec<Decl>,
    map!(braces!(many0!(parse_declaration)), |(_,r)| r)
}

