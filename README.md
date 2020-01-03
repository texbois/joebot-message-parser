# VkOpt Message Parser

[![](http://meritbadge.herokuapp.com/vkopt-message-parser)](https://crates.io/crates/vkopt-message-parser)

Tested on Rust 1.39.

## CLI Example

Extracting texts from users `id1` and `id2` from a chat dump:

```sh
cargo run --release --example cli -- -o messages.txt --only-include-names=id1,id2 -- messages.html
```

To see all available options, run:

```sh
cargo run --release --example cli -- --help
```

## API Example

See `examples/cli.rs`.

## Changelog

### 0.2.0

* Forwarded messages are now correctly parsed
* The current nesting level is reported for each message:
this can be used to reconstruct forwarded message chains or skip forwarded messages altogether
* Minor performance improvements

### 0.1.0

Initial release
