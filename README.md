# BetterDebug: Saner and cooler Debug macro for rust

[![Continuous integration](https://github.com/rakbladsvalsen/BetterDebug/actions/workflows/ci.yml/badge.svg)](https://github.com/rakbladsvalsen/BetterDebug/actions/workflows/ci.yml)
[![docs](https://docs.rs/better-debug/badge.svg)](https://docs.rs/better-debug)


This crate aims to provide a nice, and actually sane implementation of the `Debug`
trait.

This macro provides the following features, compared to the standard library `Debug`'s macro:

- Mark a field as secret (this will hide its contents when fmt::Debug is called upon it)
- Ability to use a custom formatter function for each struct field. Furthermore, you have the ability to return a `None` in your custom formatter if you want to skip printing that specific field, or if you want to use the default formatter. All of this can be configured via a macro attribute.
- Ability to prevent fields from being formatted. 
- Ability to rename any given field to whatever you want. 

## Examples

Note: You can find more examples [here](https://docs.rs/better-debug/1.0.0/better_debug/).

```rust
 use better_debug::BetterDebug;

 fn foo(foo: &Foo) -> Option<&'static str> {
     if foo.bar.len() < 5 {
         return Some("lorem ipsum");
     }
     None
 }

 #[derive(BetterDebug)]
 struct Foo {
     #[better_debug(cust_formatter = "foo")]
     bar: String,
     baz: String,
}
```

## License

MIT
