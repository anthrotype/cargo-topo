# cargo-topo 🐁

A Cargo subcommand for listing workspace crates in topological dependency order, useful for understanding build order, or scripting CI/CD pipelines.

`cargo-topo` uses the [guppy](https://crates.io/crates/guppy) crate to parse workspace metadata and perform topological sorting on the dependency graph.

Topological dependency order ensures dependencies appear before the crates that depend on them.
For example, if `api-server` depends on `core-lib`, and `core-lib` depends on `utils`, the order would be: `utils`, `core-lib`, `api-server`.
Use `--reverse` to see reverse dependency order (dependents first).

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

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE-APACHE)).
