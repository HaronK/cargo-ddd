# cargo-ddd

[![Crates.io](https://img.shields.io/crates/v/cargo-ddd.svg)](https://crates.io/crates/cargo-ddd)
[![API Docs](https://docs.rs/cargo-ddd/badge.svg)](https://docs.rs/cargo-ddd)
[![dependency status](https://deps.rs/repo/github/EmbarkStudios/cargo-ddd/status.svg)](https://deps.rs/repo/github/EmbarkStudios/cargo-ddd)

**cargo-ddd** (dependency deep diff) is a tool that generates a Git diff links (only GitHub links at the moment) for 2 versions of the crate or for all (or specified) dependencies of the workspace.

It will be usefull to inspect what changes come to the project on dependency version update or just to check chenges between 2 versions of the suspicious crate.

Inspection can be done either manually by clicking on the link or by giving it to the AI chat bot and asking for summary and suspicious changes analysis.

This can help to investigate and prevent supply chain attacks on the crates.io.

## Installation

```bash
cargo install cargo-ddd
```

## Usage

```
# cargo ddd -h
A cargo subcommand for inspecting what changes brings dependency version update into your project

Usage: cargo-ddd [OPTIONS] [CRATES]...

Arguments:
  [CRATES]...  List of crates with optional versions to inspect

Options:
  -c, --cargo-path <CARGO_PATH>        Path to `cargo` executable.  If not set, this will use the the `$CARGO` environment variable, and if that is not set, will simply be `cargo`
  -m, --manifest-path <MANIFEST_PATH>  Path to `Cargo.toml` [default: .]
  -a, --show-all                       If true then show diffs for all nested dependencies otherwise only direct ones
  -g, --group                          Group changes per direct dependency
  -v, --verbose                        Show human readable output
  -d, --diff-rs                        Generate diff links for diff.rs site instead of original one
  -h, --help                           Print help
  -V, --version                        Print version
```

To generate diff links for dependency updates in the current workspace run:
```bash
cargo ddd
```

Generating diff links for direct and all nested dependency updates:
```bash
cargo ddd -a
```

If workspace contains several dependencies that have to be updated then above command will generate a long list of changes. You can ask to generate diffs only for specific dependency:
```bash
cargo ddd -a serde
```

To see differences between current and specific (not the latest) dependency version run:
```bash
cargo ddd -a serde@1.0.225
```
or
```bash
cargo ddd -a serde@-1.0.225
```

To see diffs between 2 versions of any crate (don't have to be dependency of the current workspace):
```bash
cargo ddd -a serde@1.0.218-1.0.225
```

By default output shows direct dependencies first and consolidated dependencies after it. To group changes per direct dependency run:
```bash
cargo ddd -a -g serde
```

To see more detailed output run:
```bash
cargo ddd -v serde
```

Example:

```bash
cargo ddd serde@1.0.216-1.0.225
```
Output:
```
# serde         1.0.216 1.0.225 https://github.com/serde-rs/serde/compare/ad8dd41...1d7899d
= proc-macro2   1.0.92  1.0.101 https://github.com/dtolnay/proc-macro2/compare/acc7d36...d3188ea
= quote         1.0.37  1.0.40  https://github.com/dtolnay/quote/compare/b1ebffa...ab1e92c
= syn           2.0.90  2.0.106 https://github.com/dtolnay/syn/compare/ac5b41c...0e4bc64
= unicode-ident 1.0.14  1.0.19  https://github.com/dtolnay/unicode-ident/compare/404f1e8...dc018bf
+ serde_derive          1.0.225 https://github.com/serde-rs/serde/commit/1d7899d671c6f6155b63a39fa6001c9c48260821
```

```bash
cargo ddd -a web-sys@0.3.72-0.3.77
```
Output:
```
# web-sys                    0.3.72  0.3.77  https://github.com/rustwasm/wasm-bindgen/tree/master/crates/web-sys/compare/3a8da7c...2405ec2
= bumpalo                    3.19.0  3.16.0  https://github.com/fitzgen/bumpalo/compare/573ed78...4eeab88
= cfg-if                     1.0.4   1.0.0   https://github.com/rust-lang/cfg-if/compare/3510ca6...e60fa1e
= memchr                     2.7.6   2.7.4   https://github.com/BurntSushi/memchr/compare/9ba486e...8ad3395
= once_cell                  1.21.3  1.20.2  https://github.com/matklad/once_cell/compare/29e3d93...4fbd4a5
= proc-macro2                1.0.103 1.0.93  https://github.com/dtolnay/proc-macro2/compare/d1bf13a...83519e8
= quote                      1.0.42  1.0.38  https://github.com/dtolnay/quote/compare/bb9e7a4...0245506
= slab                       0.4.11  0.4.9   https://github.com/tokio-rs/slab/compare/2e5779f...b709dcf
= syn                        2.0.111 2.0.96  https://github.com/dtolnay/syn/compare/4e50867...d1cbce8
= unicode-ident              1.0.22  1.0.14  https://github.com/dtolnay/unicode-ident/compare/10d5e53...404f1e8
= wasm-bindgen-macro         0.2.106 0.2.100 https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro/compare/11831fb...2405ec2
= wasm-bindgen-macro-support 0.2.106 0.2.100 https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support/compare/11831fb...2405ec2
= wasm-bindgen-shared        0.2.106 0.2.100 https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared/compare/11831fb...2405ec2
+ autocfg                            1.4.0   https://github.com/cuviper/autocfg/commit/d07df6624a4573803a29397c0ccf636aa0b3d153
+ log                                0.4.22  https://github.com/rust-lang/log/commit/d5ba2cfee9b3b4ca1fcad911b7f59dc79eeee022
+ wasm-bindgen-backend               0.2.100 https://github.com/rustwasm/wasm-bindgen/tree/master/crates/backend/commit/2405ec2b4bcd1cc4e3bd1562c373e9d5f0cbdcb5
- rustversion                1.0.22          https://github.com/dtolnay/rustversion/commit/9e86f839b6a34a7d9398f243d88bf400b7fa1f7c
```

Alternatively, it's possible to generate diff links to **diff.rs** site with `-d`/`--diff-rs` flag:
```bash
cargo ddd -d -a web-sys@0.3.72-0.3.77
```
Output:
```
# web-sys                    0.3.72  0.3.77  https://diff.rs/web-sys/0.3.72/0.3.77
= bumpalo                    3.19.0  3.16.0  https://diff.rs/bumpalo/3.19.0/3.16.0
= cfg-if                     1.0.4   1.0.0   https://diff.rs/cfg-if/1.0.4/1.0.0
= memchr                     2.7.6   2.7.4   https://diff.rs/memchr/2.7.6/2.7.4
= once_cell                  1.21.3  1.20.2  https://diff.rs/once_cell/1.21.3/1.20.2
= proc-macro2                1.0.103 1.0.93  https://diff.rs/proc-macro2/1.0.103/1.0.93
= quote                      1.0.42  1.0.38  https://diff.rs/quote/1.0.42/1.0.38
= slab                       0.4.11  0.4.9   https://diff.rs/slab/0.4.11/0.4.9
= syn                        2.0.111 2.0.96  https://diff.rs/syn/2.0.111/2.0.96
= unicode-ident              1.0.22  1.0.14  https://diff.rs/unicode-ident/1.0.22/1.0.14
= wasm-bindgen-macro         0.2.106 0.2.100 https://diff.rs/wasm-bindgen-macro/0.2.106/0.2.100
= wasm-bindgen-macro-support 0.2.106 0.2.100 https://diff.rs/wasm-bindgen-macro-support/0.2.106/0.2.100
= wasm-bindgen-shared        0.2.106 0.2.100 https://diff.rs/wasm-bindgen-shared/0.2.106/0.2.100
+ autocfg                            1.4.0   https://diff.rs/autocfg/1.4.0/1.4.0
+ log                                0.4.22  https://diff.rs/log/0.4.22/0.4.22
+ wasm-bindgen-backend               0.2.100 https://diff.rs/wasm-bindgen-backend/0.2.100/0.2.100
- rustversion                1.0.22          https://diff.rs/rustversion/1.0.22/1.0.22
```

Output prefixes:
- **:** - workspace target name
- **#** - direct dependency/crate
- **=** - updated nested dependency
- **+** - added nested dependency
- **-** - removed nested dependency

> [!WARNING]
> This is an initial version that may not always extract all the necessary information and generate correct output, though crate name and versions are always correct.

## Links

- [CHANGELOG](./CHANGELOG.md)

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
