# VkOpt Message Parser

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
