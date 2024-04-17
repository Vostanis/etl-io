use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, punctuated::Punctuated, Attribute, Block, Stmt, Token, TypePath};

#[proc_macro]
pub fn etl(input: TokenStream) -> TokenStream {
    let arg = parse_macro_input!(input as Arg);

    let type1 = &arg.type_one;
    let type2 = &arg.type_two;
    let stmts = &arg.stmts;

    quote! {
        impl pipe_io::Input for #type1 {}
        impl pipe_io::Output for #type2 {}
        impl pipe_io::ETL<#type1, #type2> for pipe_io::Pipe<#type1, #type2>
        {
            #(#stmts)*
        }
    }
    .into()
}

struct Arg {
    type_one: TypePath,
    type_two: TypePath,
    stmts: Vec<Stmt>,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        // @ Input -> Output
        // input.parse::<Token![@]>()?;
        let type_one: TypePath = input.parse()?;
        input.parse::<Token![->]>()?;
        let type_two: TypePath = input.parse()?;

        // { async fn func() { ... } ... }
        let brace_content;
        let _brace_token = braced!(brace_content in input);
        let _inner_brace_attrs = brace_content.call(Attribute::parse_inner)?;
        let stmts = brace_content.call(Block::parse_within)?;

        // return 1 Arg - can this be abstracted to a Vec<Arg>?
        Ok(Arg {
            type_one,
            type_two,
            stmts,
        })
    }
}

struct Args {
    args: Vec<Arg>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let args =  Punctuated::<Arg, Token![@]>::parse_terminated(input)?;
        Ok(Args {
            args: args.into_iter().collect()
        })
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
// Test

#[proc_macro]
pub fn pipeline(input: TokenStream) -> TokenStream {

    let arg = parse_macro_input!(input as Arg);
    let type1 = &arg.type_one;
    let type2 = &arg.type_two;
    let stmts = &arg.stmts;

    quote! {
        impl Trait<#type1, #type2> for Wrapper<#type1, #type2>
        {
            #(#stmts)*
        }
    }
    .into()
}

// enum ArgInput {
//     TypePath(TypePath),
//     Type(Type),
// }

// impl Parse for ArgInput {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let lookahead = input.lookahead1();
//         if lookahead.peek(Token![::]) || lookahead.peek(Token![<]) {
//             input.parse().map(ArgInput::TypePath)
//         } else if lookahead.peek() {
//             input.parse().map(ArgInput::Type)
//         } else {
//             Err(lookahead.error())
//         }
//     }
// }