# Ameelio Service Jeeves
A collection of utilities for running web services with rust.

## Installation

Add `asjeeves = { git = "https://github.com/Ameelio/json_web_key.git", tag = 'v0.1.0' }` to your project's
Cargo.toml

## Contributing
This software is for internal use, and released under the MIT-LICENSE for educational purposes.

### Local Setup

#### Dependencies

|        |        |                                               |
| ------ | ------ | --------------------------------------------- |
| Valkey | `7.x`  | [Doc](https://valkey.io/topics/installation/) |
| Rust   | `1.86` | [Doc](https://rust-lang.org/tools/install/)   |

#### Tests

The tests require a `redis` compatible server to be running. Then you can use:
`TEST_REDIS_URL=redis://localhost:6379 cargo test`
