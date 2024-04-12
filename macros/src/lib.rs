use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, AngleBracketedGenericArguments, Attribute, Block, Ident, Stmt, Token, Type, TypePath};

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

enum ArgInput {
    WrappedType(TypePath),
    Type(Type),
}

impl Parse for ArgInput {
    fn parse(input: ParseStream) -> Result<Self> {
        
    }
}

struct Arg {
    type_one: ArgInput,
    type_two: ArgInput,
    stmts: Vec<Stmt>,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {

        // @ Input -> Output
        input.parse::<Token![@]>()?;
        let type_one: Type = input.parse()?;
        input.parse::<Token![->]>()?;
        // let type_two: Type =  input.parse()?;
        let type_two = if input.peek(syn::token::Type) & input.peek2(Token![<]) {
            input.parse::<TypePath>()?
        } else {
            input.parse::<Type>()?
        }
        
        // need to peek ahead for `<`

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