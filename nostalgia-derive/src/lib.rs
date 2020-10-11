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
    println!("{:#?}", input.attrs);

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

// Builds up a HashMap of the key/value pairs set in the attributes
// For example if you pass #[key = 'id'] as an attribute with the macro this will return
// {"key": "id"}
fn find_attr_keypairs(attrs: &Vec<syn::Attribute>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for attr in attrs {
        match attr.parse_meta().unwrap() {
            NameValue(nm) => match (nm.path.get_ident(), nm.lit) {
                (Some(ident), syn::Lit::Str(s)) => {
                    result.insert(ident.to_string(), s.value());
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
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                // Find the key field
                // Iterate over each of the fields in the struct and look for one named the same as
                // the argument passed to the key attr
                let key_field = fields
                    .named
                    .iter()
                    .find(|f| {
                        let name = &f.ident;
                        if let Some(n) = name {
                            if n.to_string() == key_values["key"] {
                                false;
                            }
                        }
                        true
                    })
                    .unwrap();

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
            }
            _ => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
