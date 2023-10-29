use darling::FromAttributes;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, FieldsNamed};

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(better_debug))]
struct FieldOptions {
    /// Whether this field should be ignored or not.
    ignore: Option<bool>,
    /// Rename the field to this name.
    rename_to: Option<String>,
    /// Whether this field is a secret or not.
    secret: Option<bool>,
    /// Custom formatter for this field. The formatter will receive a reference
    /// of the struct as an argument.
    /// The formatter should be a function that receives an instance
    /// of the struct by reference and returns an Option<T>, where T: &dyn fmt::Debug.
    /// If the formatter function returns None, then the field will use the default
    /// formatter.
    cust_formatter: Option<String>,
    /// Whether to skip formatting if the formatter returns None.
    /// Set to false by default.
    cust_formatter_skip_if_none: Option<bool>,
}

impl FieldOptions {
    /// Whether this FieldOptions is invalid. Generally, if `ignore` is set to
    /// true, then it doesn't make any sense to use any of the other options.
    fn is_invalid(&self) -> bool {
        if self.ignore.unwrap_or(false) && (self.rename_to.is_some()
                || self.secret.unwrap_or(false) || self.cust_formatter.is_some()) {
            return true;
        }
        if self.secret.unwrap_or(false) && self.cust_formatter.is_some() {
            return true;
        }
        false
    }
}

fn expand(ast: DeriveInput) -> syn::Result<TokenStream2> {
    let iden = &ast.ident;
    let fields = if let Data::Struct(DataStruct {
        fields: syn::Fields::Named(FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        abort!(iden, "BetterDebug only works with structs");
    };

    let mut out = vec![];
    for field in fields {
        let field_attributes = FieldOptions::from_attributes(&field.attrs)?;
        if field_attributes.is_invalid() {
            abort!(
                field.ident,
                "Selected options aren't compatible with each other."
            );
        }
        if field_attributes.ignore.unwrap_or(false) {
            continue;
        }
        let field_ident = match &field.ident {
            Some(ident) => ident,
            None => abort!(field, "Field must have an identifier."),
        };
        let field_name = match field_attributes.rename_to {
            Some(name) => name,
            None => field_ident.to_string(),
        };
        if let Some(func) = field_attributes.cust_formatter {
            let expr = syn::parse_str::<syn::Expr>(&func)?;
            match field_attributes
                .cust_formatter_skip_if_none
                .unwrap_or(false)
            {
                // If custom formatter returned none, skip formatting
                true => out.push(quote! {
                    if let Some(out) = #expr(&self){
                        dbg_struct.field(#field_name, &out);
                    }
                }),
                // Use default formatter if cust formatter returned None
                false => out.push(quote! {
                    if let Some(out) = #expr(&self){
                        dbg_struct.field(#field_name, &out);
                    } else {
                        dbg_struct.field(#field_name, &self.#field_ident);
                    }
                }),
            }
        } else if field_attributes.secret.unwrap_or(false) {
            out.push(quote! {
                dbg_struct.field(#field_name, &"<SECRET>");
            });
        } else {
            out.push(quote! {
                dbg_struct.field(#field_name, &self.#field_ident);
            });
        }
    }

    let ident_name = iden.to_string();

    let expanded = quote! {
        impl core::fmt::Debug for #iden {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                let mut dbg_struct = f.debug_struct(#ident_name);
                #(#out)*
                dbg_struct.finish()
            }
        }
    };

    Ok(expanded)
}

#[proc_macro_derive(BetterDebug, attributes(better_debug))]
#[proc_macro_error]
pub fn derive(input: TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    expand(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
