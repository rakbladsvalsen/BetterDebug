# BetterDebug: Saner and cooler Debug macro for rust

This crate aims to provide a nice, and actually sane implementation of the `Debug`
trait.

This macro provides the following features, compared to the standard library `Debug`'s macro:

- Mark a field as secret (this will hide its contents when fmt::Debug is called upon it)
- Ability to use a custom formatter function for each struct field. Furthermore, you have the ability to return a `None` in your custom formatter if you want to skip printing that specific field, or if you want to use the default formatter. All of this can be configured via a macro attribute.
- Ability to prevent fields from being formatted. 
- Ability to rename any given field to whatever you want. 

## Examples

```rust

#[derive(BetterDebug)]
struct Test{
    username: String,
    #[better_debug(cust_formatter="some_func", cust_formatter_skip_if_none)]
    name: String,
    #[better_debug(secret, rename_to="Super safe user's password")]
    password: String,
}

fn some_func(s: &Test) -> Option<String>{
    if s.name == String::from("123"){
        return Some("INVALID NAME!!!".into());
    }
    None
}
```


## License

MIT
