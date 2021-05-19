use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, Ident};

#[proc_macro_derive(Gc)]
pub fn gc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut generics = input.generics.clone();

    // Add `T: Trace` bounds for all type parameters.
    let where_clause = generics.make_where_clause();
    for param in input.generics.type_params() {
        let param_ident = &param.ident;
        where_clause
            .predicates
            .push(parse_quote! { #param_ident: ruffle_gc::Trace });
    }

    // An object with no lifetimes cannot contain interior GC data and doesn't need to be traced.
    let needs_trace = if generics.lifetimes().count() == 0 {
        quote! { unsafe fn needs_trace() -> bool { false } }
    } else {
        quote! {}
    };

    let ty_name = &input.ident;
    let trace_calls = match &input.data {
        Data::Struct(data) => trace_fields(&data.fields),
        Data::Enum(data) => {
            let trace_variants: Vec<_> = data
                .variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    match variant.fields.clone() {
                        Fields::Named(fields) => {
                            let ctor_fields: Vec<_> = fields
                                .named
                                .iter()
                                .map(|field| {
                                    let name = &field.ident;
                                    quote! { #name }
                                })
                                .collect();
                            let trace_fields: Vec<_>   = fields
                            .named
                            .iter()
                            .map(|field| {
                                let name = &field.ident;
                                quote! { #name.trace(ctx); }
                            })
                            .collect();
                            quote! { #ty_name::#variant_name { #(#ctor_fields),* } => { #( #trace_fields ),* } }
                        }
                        Fields::Unnamed(fields) => {
                            let ctor_fields: Vec<_> = fields
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, _)| {
                                    let name = Ident::new(&format!("field{}", i), Span::call_site());
                                    quote! { #name }
                                })
                                .collect();
                                let trace_fields: Vec<_>   = fields
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, _)| {
                                    let name = Ident::new(&format!("field{}", i), Span::call_site());
                                    quote! { #name.trace(ctx); }
                                }).collect();
                            quote! { #ty_name::#variant_name { #(#ctor_fields),* } => { #( #trace_fields ),* } }
                        }
                        Fields::Unit => {
                            quote! { #ty_name::#variant_name => (), }
                        }
                    }
                })
                .collect();
            quote! {
                match self {
                    #( #trace_variants )*
                }
            }
        }
        Data::Union(_) => panic!("Unions not supported by #[derive(Gc)]"),
    };

    let gc_lifetime_impl = lifetime(&input);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        unsafe impl #impl_generics ruffle_gc::Trace for #ty_name #ty_generics #where_clause {
            unsafe fn trace(&self, ctx: &mut ruffle_gc::GcContext) {
                #trace_calls
            }

            #needs_trace
        }

        #gc_lifetime_impl
    };

    output.into()
}

fn trace_fields(fields: &Fields) -> proc_macro2::TokenStream {
    let trace_calls: Vec<_> = match fields {
        Fields::Unit => Vec::new(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, _field)| {
                let i = syn::Index::from(i);
                quote! {
                    self.#i.trace(ctx);
                }
            })
            .collect(),
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let name = field.ident.clone().unwrap();
                quote! { self.#name.trace(ctx); }
            })
            .collect(),
    };
    quote! {
        #( #trace_calls )*
    }
}

fn lifetime(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ty_name = input.ident.clone();
    let mut generics = input.generics.clone();
    let num_lifetimes = generics.lifetimes().count();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    if num_lifetimes == 0 {
        let out = quote! {
            unsafe impl ruffle_gc::GcLifetime<'_> for #ty_name {
                type Aged = Self;
            }
        };
        return out.into();
    } else if num_lifetimes > 1 {
        panic!("Don't know how to deal with multiple lifetimes")
    }

    generics.params.push(parse_quote! { '_lt });
    //let trait_lifetime = quote! { '_lt };

    let where_clause = generics.make_where_clause();

    for param in input.generics.type_params() {
        let ty = &param.ident;
        where_clause.predicates.push(parse_quote! { #ty: '_lt  });
    }

    let mut aged_generics = input.generics.clone();
    if let Some(lifetime) = aged_generics.lifetimes_mut().next() {
        lifetime.lifetime.ident = parse_quote! { _lt };
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, aged_ty_generics, _) = aged_generics.split_for_impl();
    quote! {
        unsafe impl #impl_generics ruffle_gc::GcLifetime<'_lt> for #ty_name #ty_generics #where_clause {
            type Aged = #ty_name #aged_ty_generics;
        }
    }
}
