# simple-octree

simple-octree aims to provide a simple and generic octree that is ideally
performant, without making assumptions about the underlying data or management
of data.

## Current features

`Octree` - a generic octree with generic functions for getting/adding/removing
child nodes. Convenience functions are provided for accessing children based on
positive/negative axis values.

## Planned features

* A managed octree type that will automatically add/remove child nodes based on
  defined limits.

## Formatting Code

NOTE: Currently the code formatting rules rely on nightly Rust. Everything else
      should compile with stable Rust.

To install rustfmt:

```bash
rustup component add rustfmt
```

To format the code:

```bash
cargo +nightly fmt
```

## License

MIT license, see LICENCE.md
