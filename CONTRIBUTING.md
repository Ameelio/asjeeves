This software is for internal use, and released under the MIT-LICENSE for educational purposes.

## Local Setup

### Dependencies

|        |        |                                               |
| ------ | ------ | --------------------------------------------- |
| Valkey | `7.x`  | [Doc](https://valkey.io/topics/installation/) |
| Rust   | `1.86` | [Doc](https://rust-lang.org/tools/install/)   |

### Tests

The tests require a `redis` compatible server to be running. Then you can use:
`TEST_REDIS_URL=redis://localhost:6379 cargo test`
