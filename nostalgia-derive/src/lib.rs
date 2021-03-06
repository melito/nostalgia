use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{parse_macro_input, Data, DeriveInput, Meta::NameValue};

#[proc_macro_derive(Storable, attributes(key, db_name))]
pub fn storable_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let name_str = name.to_string();
    let key_definition = find_key_name_and_type(&input.attrs, &input.data);

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl Record for #name {
            #key_definition

            fn db_name() -> &'static str {
                #name_str
            }
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

fn find_attr_keypairs(attrs: &Vec<syn::Attribute>) -> HashMap<String, syn::LitStr> {
    let mut result = HashMap::new();
    for attr in attrs {
        match attr.parse_meta().unwrap() {
            NameValue(nm) => match (nm.path.get_ident(), nm.lit) {
                (Some(ident), syn::Lit::Str(s)) => {
                    result.insert(ident.to_string(), s);
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    result
}

fn find_key_name_and_type(attrs: &Vec<syn::Attribute>, data: &syn::Data) -> TokenStream {
    let key_values = find_attr_keypairs(attrs);
    let fuck = find_attr_keypairs(attrs);
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                if let Some(key_field) = find_key_name_in_struct(fields, key_values) {
                    match (key_field.ident.as_ref(), key_field.ty.clone()) {
                        (Some(ident), syn::Type::Path(type_path)) => {
                            let prop = ident;
                            let prop_type = type_path.path.get_ident().unwrap();

                            quote! {
                                type Key = Key<#prop_type>;

                                fn key(&self) -> Self::Key {
                                    Key::from(self.#prop)
                                }
                            }
                        }
                        _ => unimplemented!(),
                    }
                } else {
                    let id = &fuck["key"];

                    return syn::Error::new(id.span(), "This field does not exist on the type")
                        .to_compile_error();
                }
            }
            _ => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

// Find the key field
// Iterate over each of the fields in the struct and look for one named the same as
// the argument passed to the key attr
fn find_key_name_in_struct(
    target_fields: &syn::FieldsNamed,
    config: HashMap<String, syn::LitStr>,
) -> Option<&syn::Field> {
    target_fields.named.iter().find(|f| {
        let name = &f.ident;
        if let Some(n) = name {
            if n.to_string() == config["key"].value() {
                return true;
            }
        }
        return false;
    })
}
