extern crate proc_macro;

use proc_macro::TokenStream;
use std::fmt::Debug;

use quote::{quote, ToTokens};
use syn;
use syn::{Attribute, Data, DeriveInput, Field, parenthesized, parse_macro_input, Path};

struct EnumMember
{
    path: Path
}

impl EnumMember
{
    fn has_enum_name(&self, path: &Path) -> bool
    {
        path.segments.iter().zip(self.path.segments.iter()).all(|(a, b)| {
            a.ident == b.ident
        })
    }
}

impl Debug for EnumMember
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let segments = self.path.segments.iter().map(|segment| {
            segment.ident.to_string()
        }).collect::<Vec<_>>();
        segments.join("::").fmt(f)
    }
}

impl ToTokens for EnumMember
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.path.to_tokens(tokens);
    }
}

impl syn::parse::Parse for EnumMember
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let path: Path = content.parse()?;
        let segments: Vec<_> = path.segments.iter().map(|segment| {
            segment.ident.to_string()
        }).collect();

        if segments.len() < 2 {
            return Err(syn::Error::new_spanned(path, "Expected enum member to be of the form `Enum::Member`"));
        }

        Ok(Self { path })
    }
}

fn get_inject_attrib_index(field: &Field) -> Option<usize>
{
    field.attrs.iter().position(|attr| {
        if let Some(ident) = attr.path.get_ident() {
            return ident == "inject";
        }
        else {
            false
        }
    })
}

fn parse_injected_fields(ast: &mut DeriveInput) -> Vec<(&Field, Attribute)> {
    let mut fields = vec![];

    if let Data::Struct(data) = &mut ast.data {
        for field in data.fields.iter_mut() {
            if let Some(i) = get_inject_attrib_index(field) {
                let attrib = field.attrs.remove(i);
                fields.push((&*field, attrib));
            }
        }
    }
    else {
        quote!(syn::Error::new_spanned(ast, "Injectable can only be applied to structs").to_compile_error().into());
    }

    fields
}

#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_val = parse_macro_input!(attr as Path);
    let mut ast = parse_macro_input!(input as DeriveInput);
    let fields: Vec<_> = parse_injected_fields(&mut ast);
    let mut matches = core::default::Default::default();

    for (field, attrib) in fields.into_iter() {
        let field_name = field.ident.as_ref().unwrap();
        // TODO: Find enum member that matches the field type

        let tokens = attrib.tokens.into();
        let member = parse_macro_input!(tokens as EnumMember);


        if !member.has_enum_name(&enum_val) {
            return quote!(syn::Error::new_spanned(ast, "All injected fields must be from the same enum").to_compile_error().into()).into();
        }

        matches = quote! {
            #matches
            #member(value) => self.#field_name = Injected::from(value),
        };
    }

    let name = &ast.ident;
    let quote = quote! {
        #ast

        impl ::injectiny::Injectable<#enum_val> for #name {
            fn inject(&mut self, model: #enum_val) {
                match model {
                    #matches
                    _ => {}
                }
            }
        }
    };
    quote.into()
}
