# token-list

[![Latest Version]][crates.io] [![Docs]][docs.rs] [![CI]][github-actions]

[CI]: https://github.com/telcoin/token-list/workflows/CI/badge.svg
[github-actions]: https://github.com/telcoin/token-list/actions
[latest version]: https://img.shields.io/crates/v/token-list.svg
[crates.io]: https://crates.io/crates/token-list
[docs]: https://docs.rs/token-list/badge.svg
[docs.rs]: https://docs.rs/token-list

Ethereum [token list] standard created by Uniswap

## usage

`Cargo.toml`:

```toml
token-list = { version = "0.1.0", features = ["from-uri"] }
tokio = { version = "1", features = ["full"] }
```

`main.rs`:

```rust
use token_list::TokenList;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token_list = TokenList::from_uri("https://defi.cmc.eth.link").await?;

    assert_eq!(token_list.name, "CMC DeFi");

    Ok(())
}
```

You don't need the `tokio` dependency if you do not wish to enable the `from_uri` method.

[token list]: https://tokenlists.org/
