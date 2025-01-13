use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, Data, DeriveInput, Field, GenericParam,
    Generics,
};

#[proc_macro_derive(Read)]
#[proc_macro_error]
pub fn derive_read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).expect("A syn parseable token stream");
    let derived = impl_read(&input);
    derived.into()
}

#[proc_macro_derive(Write)]
#[proc_macro_error]
pub fn derive_write(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).expect("A syn parseable token stream");
    let derived = impl_write(&input);
    derived.into()
}

fn impl_read(input: &DeriveInput) -> proc_macro2::TokenStream {
    use syn::DataStruct;

    let struct_identifier = &input.ident;
    let data = &input.data;
    let generics = input.generics.clone();
    match data {
        Data::Struct(DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => impl_read_for_struct(struct_identifier, generics, &fields.named),
        _ => {
            abort_call_site!("Deriving Services is only supported for named Structs");
        }
    }
}

fn impl_read_for_struct(
    identifier: &proc_macro2::Ident,
    generics: Generics,
    fields: &Punctuated<Field, Comma>,
) -> proc_macro2::TokenStream {
    let fields_read = fields.iter().map(|field| {
        let field_identifier = field.ident.as_ref().expect("A struct attribute identifier");
        let field_ty = &field.ty;

        quote! {
            #field_identifier: <#field_ty>::read()
        }
    });

    // Add a bound `T: Read` to every type parameter T.
    // let generics = add_read_trait_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics risc0_zkvm_io::Read for #identifier #ty_generics #where_clause {
            fn read() -> Self {
                Self {
                    #( #fields_read ),*
                }
            }
        }
    }
}

// // Add a bound `T: Read` to every type parameter T.
// fn add_read_trait_bounds(mut generics: Generics) -> Generics {
//     for param in &mut generics.params {
//         if let GenericParam::Type(ref mut type_param) = *param {
//             type_param.bounds.push(parse_quote!(risc0_zkvm_io::Read));
//         }
//     }
//     generics
// }

// // Add a bound `T: Write` to every type parameter T.
// fn add_write_trait_bounds(mut generics: Generics) -> Generics {
//     for param in &mut generics.params {
//         if let GenericParam::Type(ref mut type_param) = *param {
//             type_param.bounds.push(parse_quote!(risc0_zkvm_io::Write));
//         }
//     }
//     generics
// }

fn impl_write(input: &DeriveInput) -> proc_macro2::TokenStream {
    use syn::DataStruct;

    let struct_identifier = &input.ident;
    let data = &input.data;
    let generics = input.generics.clone();
    match data {
        Data::Struct(DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => impl_write_for_struct(struct_identifier, generics, &fields.named),
        _ => {
            abort_call_site!("Deriving Services is only supported for named Structs");
        }
    }
}

fn impl_write_for_struct(
    identifier: &proc_macro2::Ident,
    generics: Generics,
    fields: &Punctuated<Field, Comma>,
) -> proc_macro2::TokenStream {
    let fields_write = fields.iter().map(|field| {
        let field_identifier = field.ident.as_ref().expect("A struct attribute identifier");
        let field_ty = &field.ty;

        quote! {
            #field_identifier: <#field_ty>::write()
        }
    });

    // Add a bound `T: Write` to every type parameter T.
    // let generics = add_write_trait_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #[cfg(not(target_os = "zkvm"))]
        impl #impl_generics risc0_zkvm_io::Write for #identifier #ty_generics #where_clause {
            fn write(&self) -> Self {
                Self {
                    #( #fields_write ),*
                }
            }
        }
    }
}
