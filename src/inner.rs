use darling::FromAttributes;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, FieldsNamed};

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
        if self.ignore.unwrap_or(false)
            && (self.rename_to.is_some()
                || self.secret.unwrap_or(false)
                || self.cust_formatter.is_some()
                || self.cust_formatter_skip_if_none.unwrap_or(false))
        {
            return true;
        }

        // Fail if cust_formatter_skip_if_none was set to true but there's no custom
        // formatter.
        if self.cust_formatter.is_none() && self.cust_formatter_skip_if_none.unwrap_or(false) {
            return true;
        }

        // Fail if secret was set to true and a custom formatter is being used.
        if self.secret.unwrap_or(false) && self.cust_formatter.is_some() {
            return true;
        }
        false
    }
}

pub(crate) fn expand(ast: DeriveInput) -> syn::Result<TokenStream2> {
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

// Tests
#[cfg(test)]
mod tests {

    use super::*;
    use syn::parse_quote;

    // Test drop-in replacement.
    #[test]
    fn test_expand() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                bar: String,
                baz: String,
            }
        };
        let expected = quote! {
            impl core::fmt::Debug for Foo {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut dbg_struct = f.debug_struct("Foo");
                    dbg_struct.field("bar", &self.bar);
                    dbg_struct.field("baz", &self.baz);
                    dbg_struct.finish()
                }
            }
        };
        let expanded = expand(input).unwrap();
        assert_eq!(expanded.to_string(), expected.to_string());
    }

    //T Test rename
    #[test]
    fn test_expand_rename() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(rename_to = "new_name")]
                bar: String,
                baz: String,
            }
        };
        let expected = quote! {
            impl core::fmt::Debug for Foo {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut dbg_struct = f.debug_struct("Foo");
                    dbg_struct.field("new_name", &self.bar);
                    dbg_struct.field("baz", &self.baz);
                    dbg_struct.finish()
                }
            }
        };
        let expanded = expand(input).unwrap();
        assert_eq!(expanded.to_string(), expected.to_string());
    }
    #[test]
    fn test_expand_rename_secret() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(rename_to = "new_name", secret)]
                bar: String,
                baz: String,
            }
        };
        let expected = quote! {
            impl core::fmt::Debug for Foo {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut dbg_struct = f.debug_struct("Foo");
                    dbg_struct.field("new_name", &"<SECRET>");
                    dbg_struct.field("baz", &self.baz);
                    dbg_struct.finish()
                }
            }
        };
        let expanded = expand(input).unwrap();
        assert_eq!(expanded.to_string(), expected.to_string());
    }
    #[test]
    fn test_expand_secret() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(secret)]
                bar: String,
                baz: String,
            }
        };
        let expected = quote! {
            impl core::fmt::Debug for Foo {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut dbg_struct = f.debug_struct("Foo");
                    dbg_struct.field("bar", &"<SECRET>");
                    dbg_struct.field("baz", &self.baz);
                    dbg_struct.finish()
                }
            }
        };
        let expanded = expand(input).unwrap();
        assert_eq!(expanded.to_string(), expected.to_string());
    }
    #[test]
    fn test_expand_cust_formatter() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(cust_formatter = "foo")]
                bar: String,
                baz: String,
            }
        };
        let expected = quote! {
            impl core::fmt::Debug for Foo {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut dbg_struct = f.debug_struct("Foo");
                    if let Some(out) = foo(&self){
                        dbg_struct.field("bar", &out);
                    } else {
                        dbg_struct.field("bar", &self.bar);
                    }
                    dbg_struct.field("baz", &self.baz);
                    dbg_struct.finish()
                }
            }
        };
        let expanded = expand(input).unwrap();
        assert_eq!(expanded.to_string(), expected.to_string());
    }
    #[test]
    fn test_expand_cust_formatter_skip_if_none() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(cust_formatter = "foo", cust_formatter_skip_if_none)]
                bar: String,
                baz: String,
            }
        };
        let expected = quote! {
            impl core::fmt::Debug for Foo {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut dbg_struct = f.debug_struct("Foo");
                    if let Some(out) = foo(&self){
                        dbg_struct.field("bar", &out);
                    }
                    dbg_struct.field("baz", &self.baz);
                    dbg_struct.finish()
                }
            }
        };
        let expanded = expand(input).unwrap();
        assert_eq!(expanded.to_string(), expected.to_string());
    }
    #[test]
    fn test_expand_ignore() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(ignore)]
                bar: String,
                baz: String,
            }
        };
        let expected = quote! {
            impl core::fmt::Debug for Foo {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut dbg_struct = f.debug_struct("Foo");
                    dbg_struct.field("baz", &self.baz);
                    dbg_struct.finish()
                }
            }
        };
        let expanded = expand(input).unwrap();
        assert_eq!(expanded.to_string(), expected.to_string());
    }

    #[test]
    #[should_panic]
    fn test_invalid_ignore_cust_formatter() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(ignore, cust_formatter = "foo")]
                bar: String,
                baz: String,
            }
        };
        expand(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_ignore_rename_to() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(ignore, rename_to = "foo")]
                bar: String,
                baz: String,
            }
        };
        expand(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_secret() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(ignore, secret)]
                bar: String,
                baz: String,
            }
        };
        expand(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_secret_cust_formatter() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(secret, cust_formatter = "foo")]
                bar: String,
                baz: String,
            }
        };
        expand(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_cust_formatter_option() {
        let input = parse_quote! {
            #[derive(BetterDebug)]
            struct Foo {
                #[better_debug(cust_formatter_skip_if_none)]
                bar: String,
                baz: String,
            }
        };
        expand(input).unwrap();
    }
}
