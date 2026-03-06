use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, FieldsNamed, parse_macro_input};

/// Provide a like-for-like implementation of ['io::manager::postgres::PostgreSQL`].
/// Take the following:
///
/// ```rust
/// #[derive(SqlMap)]
/// struct MyStruct {
///     field0: String,
///     field1: i64,
/// }
/// ```
///
/// Above is equivalent to below:
///
/// ```rust
/// struct MyStruct {
///     field0: String,
///     field1: i64,
/// }
///
/// impl datax_io::manager::postgres::PostgreSQL for &MyStruct {
///     fn sql_map(&self) -> std::vec::Vec<&(dyn datax_io::ToSql + std::marker::Sync)> {
///         std::vec::Vec::from([
///             &self.field0,
///             &self.field1,
///         ])
///     }
/// }
/// ```
#[proc_macro_derive(PostgreSQL)]
pub fn derive_postgresql(item: TokenStream) -> TokenStream {
    let body = parse_macro_input!(item as DeriveInput);

    // Extract the struct name.
    let struct_name = &body.ident;

    // Extract field names.
    let fields = match &body.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(FieldsNamed { named, .. }) => named,
            _ => panic!("PostgreSQL can only be derived for structs with named fields"),
        },
        _ => panic!("PostgreSQL can only be derived for structs"),
    };

    // Create an array of references to each field.
    let field_refs = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        // Check if the type is OrderedFloat
        let type_string = quote!(#field_type).to_string();
        if type_string.contains("OrderedFloat") {
            quote! { &self.#field_name.0 } // Extract inner f64
        } else {
            quote! { &self.#field_name }
        }
    });

    // Return the implementation.
    quote! {
        impl petl::manager::postgres::PostgreSQL for &#struct_name {
            fn sql_map(&self) -> std::vec::Vec<&(dyn petl::ToSql + std::marker::Sync)> {
                vec![#(#field_refs),*]
            }
        }
    }
    .into()
}
