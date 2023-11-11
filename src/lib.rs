//! Drop-in replacement for `#[derive(Debug)]`
//!
//! Have you used `#[derive(Debug)]` on a struct?
//! Well, you've probably noticed that it prints out all the fields of the struct,
//! even the ones that you don't want to print out. If you have a `Vec<T>` in your
//! struct, forget about printing it because that will clutter the logs very badly.
//!
//! Or maybe you just want to customize the name of a field, or avoid printing it
//! completely?
//!
//! Look no more! `BetterDebug` is a drop-in replacement for `#[derive(Debug)]` that
//! allows you to customize the behavior of the `Debug` macro.
//!
//! This crate provides an extremely simple, fast, efficient and no-overhead
//! replacement for the standard library's Debug macro. BetterDebug allows
//! you to skip fields from being printed, mark them as a secret, use a custom
//! formatter, etc.
//!
//! Please see the documentation below for usage instructions.
//!
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod inner;

/// # Macro usage
///
/// Add the following to your Cargo.toml:
///
/// ```toml
/// [dependencies]
/// better_debug = "1.0"
/// ```
///
/// Then, add the following to your program:
///
/// ```rust
/// use better_debug::BetterDebug;
///
/// #[derive(BetterDebug)]
/// struct Person {
///     name: String,
///     age: u8,
///     secret: String,
/// }
/// ```
///
/// The above code will implement `fmt::Debug` just like the standard library's
/// `Debug` macro. Essentially, you've made no changes.
///
/// # Cookbook recipes
///
/// ## Ignore a custom struct member
///
/// This will completely prevent `bar` from being printed.
///
/// ```rust
/// use better_debug::BetterDebug;
///
/// #[derive(BetterDebug)]
/// struct Foo {
///     #[better_debug(ignore)]
///     bar: String,
///     baz: String,
/// }
/// ```
///
/// ## Rename a field
///
/// This will print `bar` as if it were `new_name`.
/// Note that you can use just about anything, i.e.
/// `bar`, `Nice Bar!`, etc.
///
/// ```rust
/// use better_debug::BetterDebug;
///
/// #[derive(BetterDebug)]
/// struct Foo {
///     #[better_debug(rename_to = "new_name")]
///     bar: String,
///     baz: String,
///}
/// ```
///
/// ## Mark a field as a secret
///
/// This will set this field's contents to
/// `<SECRET>`, regardless of its actual contents.
///
/// ```rust
/// use better_debug::BetterDebug;
///
/// #[derive(BetterDebug)]
/// struct Foo {
///     #[better_debug(secret)]
///     bar: String,
///     baz: String,
///}
/// ```
/// ## Use a custom formatter.
///
/// You can use a custom function to format the contents of a field.
///
/// This function must take a reference to the entire struct and return
/// `Some(dyn fmt::Debug)` or None to use the default formatter.
/// You can also return None to prevent the field from being printed.
///
/// Note that there's no hard requirement in the return type of the function:
/// you can return any `Some(T)` as long as `T` is printable, i.e. it implements
/// `fmt::Debug`. The examples below use `&'static str` for convenience.
///
/// ### Use a custom formatter with fallback
///
/// ```rust
/// use better_debug::BetterDebug;
///
/// fn foo(foo: &Foo) -> Option<&'static str> {
///     if foo.bar.len() < 5 {
///         return Some("lorem ipsum");
///     }
///     None
/// }
///
/// #[derive(BetterDebug)]
/// struct Foo {
///     #[better_debug(cust_formatter = "foo")]
///     bar: String,
///     baz: String,
///}
/// ```
///
/// ### Use a custom formatter without fallback
///
/// ```rust
/// use better_debug::BetterDebug;
///
/// fn foo(foo: &Foo) -> Option<&'static str> {
///     if foo.bar != "lorem ipsum" {
///         // If bar isn't equal to "lorem ipsum", then
///         // don't print anything at all.
///         return None;
///     }
///     Some("lorem ipsum is great")
/// }
///
/// #[derive(BetterDebug)]
/// struct Foo {
///     #[better_debug(cust_formatter = "foo", cust_formatter_skip_if_none)]
///     bar: String,
///     baz: String,
///}
/// ```
#[proc_macro_derive(BetterDebug, attributes(better_debug))]
#[proc_macro_error]
pub fn derive(input: TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    inner::expand(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
