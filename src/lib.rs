//! Macro for dealing with process boilerplate.
extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;
extern crate synstructure;

use proc_macro::TokenStream;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse::Error, parse_quote, punctuated::Punctuated, spanned::Spanned, FnArg, Generics, Ident,
    ImplItem, ImplItemMethod, ItemImpl, ReturnType, Token, Type, TypeTuple,
};

#[proc_macro_attribute]
pub fn auto_from(attributes: TokenStream, input: TokenStream) -> TokenStream {
    match actual_process_attribute(attributes, input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}

struct FromImplBlock {
    self_type: Type,
    functions: Vec<ProcessedFromHandler>,
}

struct ParsedFromHandler {
    message_type: Type,
    original_method: ImplItemMethod,
}

struct ProcessedFromHandler {
    message_type: Type,
    generics: Generics,
    recv_method: ImplItemMethod,
}

fn actual_process_attribute(
    _attributes: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, Error> {
    let input: ItemImpl = syn::parse(input)?;

    let interpreted = parse_impl_block(input)?;

    let output = generate_output(interpreted)?;

    Ok(output)
}

fn parse_impl_block(input: ItemImpl) -> Result<FromImplBlock, Error> {
    if let Some(unsafety) = input.unsafety {
        return Err(Error::new(
            unsafety.span(),
            "unexpected 'unsafe impl' in #[auto_from] impl block",
        ));
    }
    if let Some((_bang, path, _for)) = input.trait_ {
        return Err(Error::new(
            path.span(),
            "expected direct impl, but found a trait implementation",
        ));
    }
    let self_type = *input.self_ty.clone();

    let mut functions = Vec::new();
    for item in &input.items {
        match item {
            ImplItem::Method(method) => {
                let mut method = method.clone();
                functions.push(process_from_handler(interpret_from_fn(method)?));
            }
            _ => {
                return Err(Error::new(
                    item.span(),
                    "unexpected non-function item in #[auto_from] impl block",
                ));
            }
        }
    }

    Ok(FromImplBlock {
        self_type,
        functions,
    })
}

fn unit_type() -> Type {
    let x: TypeTuple = parse_quote!(());
    x.into()
}

fn interpret_from_fn(item: ImplItemMethod) -> Result<ParsedFromHandler, Error> {
    let inputs = &item.sig.decl.inputs;
    if inputs.len() != 1 {
        return Err(Error::new(
            inputs.span(),
            "expected function taking one arguments: the message",
        ));
    }
    let message_type = get_fn_arg_type(inputs.iter().next().unwrap())?;

    Ok(ParsedFromHandler {
        message_type,
        original_method: item,
    })
}

fn process_from_handler(data: ParsedFromHandler) -> ProcessedFromHandler {
    let ParsedFromHandler {
        message_type,
        original_method: mut method,
    } = data;

    let generics = method.sig.decl.generics.clone();
    method.sig.ident = Ident::new("from", Span::call_site());
    method.sig.decl.generics.where_clause = None;
    method.sig.decl.generics.params = Punctuated::new();
    return ProcessedFromHandler {
        message_type,
        generics,
        recv_method: method,
    };
}

fn get_fn_arg_type(arg: &FnArg) -> Result<Type, Error> {
    match arg {
        FnArg::Captured(arg) => Ok(arg.ty.clone()),
        FnArg::Ignored(ty) => Ok(ty.clone()),
        other => Err(Error::new(
            other.span(),
            "expected non-self argument with type",
        )),
    }
}

fn get_fn_return_type(ty: &ReturnType) -> Type {
    match ty {
        ReturnType::Default => unit_type(),
        ReturnType::Type(_, ty) => (**ty).clone(),
    }
}

fn generate_output(block: FromImplBlock) -> Result<TokenStream, Error> {
    let FromImplBlock {
        self_type,
        functions,
    } = block;

    let mut tokens = TokenStream2::new();

    // impl MessageReceiver
    for ProcessedFromHandler {
        message_type,
        generics,
        recv_method,
    } in &functions
    {
        let (impl_generics, _, where_generics) = generics.split_for_impl();

        (quote! {
            impl #impl_generics ::std::convert::From<#message_type> for #self_type
            #where_generics
            {
                #recv_method
            }
        })
        .to_tokens(&mut tokens);
    }

    Ok(tokens.into())
}
