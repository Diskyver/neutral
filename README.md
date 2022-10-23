# neutral - unofficial rust client for neutrinoapi.com

[![crate.io-badge](https://img.shields.io/badge/crate.io-neutral-orange)](https://crates.io/crates/neutral)
[![Rust](https://github.com/Diskyver/neutral/actions/workflows/rust.yaml/badge.svg)](https://github.com/Diskyver/neutral/actions/workflows/rust.yaml)
[![documentation bagde](https://img.shields.io/badge/doc.rs-latest-blue)](https://docs.rs/neutral/latest/neutral/index.html)

Provide an API to interact with some features provided by [neutrinoapi.com](https://www.neutrinoapi.com).

# What is neutrinoapi.com
A general-purpose tool that solves recurring problems encountered during the development of software systems. It is used across many industries by software developers, data scientists and systems operators.

# How to use the neutral crate ?
The [Neutral](./struct.Neutral.html) structure act as an API client of neutrinoapi.
Features are represented by modules, each module contains a struct which implement a `send` method to call neutrinoapi.com. Use an instance of [Neutral](./struct.Neutral.html) to interact with neutrinoapi.


Example for ip_info endpoint:

```rust
let api_auth = ApiAuth::new("userid".to_string(), "apikey".to_string());
let neutral = Neutral::try_new("https://neutrinoapi.net", api_auth).unwrap();
let ip_info_response = neutral.ip_info().send(ip_addr).await.unwrap();
```