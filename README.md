# cargo-real-deps

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## About

Cargo subcommand to check which crates are built when building for the host machine with the selected features

The `Cargo.lock` file indicates the entire list of dependencies, but some of those might not be built depending on the platform or the set of features you have chosen.
This tool will give you the actual list of dependencies, their version and the activated features depending on how you build it

As an example, here is the dependency list generated for a normal build of [nom](https://lib.rs/nom):

```
$ cargo real-deps nom/Cargo.toml
cfg-if 0.1.9 {}
semver-parser 0.7.0 {}
semver 0.9.0 {"default"}
rustc_version 0.2.3 {}
ryu 1.0.0 {}
void 1.0.2 {}
unreachable 1.0.0 {}
stackvector 1.0.6 {"std", "default"}
static_assertions 0.3.4 {}
lexical-core 0.4.3 {"correct", "ryu", "stackvector", "table", "std", "default"}
memchr 2.2.1 {"use_std"}
version_check 0.1.5 {}
nom 5.0.1 {"default", "std", "lexical", "lexical-core", "alloc"}
```

But if you wanted to see which dependencies are built when in "no std":

```
$ cargo real-deps nom/Cargo.toml --no-default-features
memchr 2.2.1 {}
version_check 0.1.5 {}
nom 5.0.1 {}
```

You can also specify exactly the features you want with the option `--features=feature1,feature2,etc`

## Installing

`cargo-real-deps` can be installed with `cargo install`:

```
cargo install cargo-real-deps
```

## Usage

```
cargo-real-deps  
Cargo subcommand to check which crates are built depending on features

USAGE:
    cargo real-deps [FLAGS] [OPTIONS] [path]

FLAGS:
        --all-features           Activate all features
    -c, --count                  Prints only the total number of dependencies
    -h, --help                   Prints help information
        --no-default-features    Deactivate default features
    -V, --version                Prints version information

OPTIONS:
        --features <features>    Activates some features

ARGS:
    <path>    Project directory, or path to Cargo.toml
```
