# neutral - unofficial rust client for neutrinoapi.com

[![crate.io-badge](https://img.shields.io/badge/crate.io-neutral-blue)](https://crates.io/crates/neutral)
[![Rust](https://github.com/Diskyver/neutral/actions/workflows/rust.yaml/badge.svg)](https://github.com/Diskyver/neutral/actions/workflows/rust.yaml)

Provide an API to interact with some features provided by [neutrinoapi.com](https://www.neutrinoapi.com).

# What is neutrinoapi.com
A general-purpose tool that solves recurring problems encountered during the development of software systems. It is used across many industries by software developers, data scientists and systems operators. 


# How to use the neutral crate ? 
The API is describe the the [Neutral](./struct.Neutral.html) structure. 
Features are represented by module, each module contains a `send` function which call neutrinoapi.com using an instance of [Neutral](./struct.Neutral.html) structure.

By example, ip info feature from neutrinoapi is implemented inside the [neutral::ip_info](./ip_info/index.html) module.

```rust
let api_auth = ApiAuth::new("userid", "apikey");
let client = Neutral::try_new("https://neutrinoapi.net/ip-info", api_auth).unwrap();
let ip_info_response = ip_info::send(&client, ip_addr).await.unwrap();
```