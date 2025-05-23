# cargo-topo

A Cargo subcommand for listing workspace crates in topological dependency order.
It can be used for understanding build order, scripting CI/CD pipelines, and analyzing complex workspace dependencies.

`cargo-topo` uses the [guppy](https://crates.io/crates/guppy) crate to parse workspace metadata, then performs topological sorting on the dependency graph.

## Usage

List workspace members:

```sh
cargo topo
```

List workspace members in reverse topological order:

```sh
cargo topo --reverse
```

List all dependencies (including external dependencies):

```sh
cargo topo --all
```

Compact output for scripting:

```sh
cargo topo --compact
```

Show dependencies for a specific package:

```sh
cargo topo --package <package_name>
```

Exclude specific packages from the output:

```sh
cargo topo --exclude <package_name>
```

Include dev-dependencies in analysis:

```sh
cargo topo --include-dev
```

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).