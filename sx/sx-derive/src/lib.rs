use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, GenericParam, Generics, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn skip_attr(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

#[proc_macro_derive(SXRepr)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_traint_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let sexpr = make_sexpr(&name, &input.data);

    let expanded = quote! {
        impl #impl_generics sx::SXRepr for #name #ty_generics #where_clause {
            #[allow(unknown_lints, clippy)]
            fn sx(self) -> sx::SX {
                #sexpr
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn add_traint_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(sx::SXRepr));
            type_param.bounds.push(parse_quote!(Clone));
        }
    }
    generics
}

fn named_fields_to_sx(fields: &syn::FieldsNamed) -> TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! { f.span() =>
            sx::SX::Field(stringify!{#name}.to_string(), Box::new(#name.clone().sx()))
        }
    });
    quote! {
        sx::SX::Expr(vec![
            #(#recurse,)*
        ])
    }
}

fn unnamed_fields_to_sx(fields: &syn::FieldsUnnamed) -> TokenStream {
    let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
        let ident = syn::Ident::new(&format!("v{i}"), f.span());
        quote_spanned! {f.span()=>
            #ident.clone().sx()
        }
    });
    quote! {
        sx::SX::Expr(vec![
            #(#recurse,)*
        ])
    }
}

fn make_sexpr(data_name: &syn::Ident, data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_vars = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! {f.span() =>
                        let #name = &self.#name;
                    }
                });
                let expr = named_fields_to_sx(fields);
                quote! {
                    #(#field_vars)*
                    #expr
                }
            }
            Fields::Unnamed(fields) => {
                let field_vars = fields.unnamed.iter().enumerate().map(|(id, f)| {
                    let varname = syn::Ident::new(&format!("v{id}"), f.span());
                    let index = syn::Index::from(id);
                    quote_spanned! {f.span() =>
                        let #varname = self.#index;
                    }
                });
                let expr = unnamed_fields_to_sx(fields);
                quote! {
                    #(#field_vars)*
                    #expr
                }
            }
            Fields::Unit => quote!(sx::SX::Nil),
        },
        Data::Enum(data) => {
            let recurse = data.variants.iter().map(|v| {
                let name = &v.ident;
                match &v.fields {
                    Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| &f.ident);
                        let expr = named_fields_to_sx(fields);
                        quote_spanned! {v.span() =>
                            #data_name::#name {#(#field_names,)*} => sx::SX::Field(
                                stringify!{#name}.to_string(), Box::new(#expr)
                            )
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_names = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(id, f)| syn::Ident::new(&format!("v{id}"), f.span()));
                        let expr = unnamed_fields_to_sx(fields);
                        quote_spanned! {v.span() =>
                            #data_name::#name (#(#field_names,)*) => sx::SX::Field(
                                stringify!{#name}.to_string(), Box::new(#expr)
                            )
                        }
                    }
                    Fields::Unit => quote_spanned! {v.span() =>
                        #data_name::#name => sx::SX::Atom(stringify!{#name}.to_string())
                    },
                }
            });
            quote! {
                match &self {
                    #(#recurse,)*
                }
            }
        }
        Data::Union(..) => unimplemented!(),
    }
}
