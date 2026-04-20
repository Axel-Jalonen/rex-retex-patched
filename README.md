# ReX - Typesetting Mathematics (Patched Fork)

**This is a mechanically patched fork of [ReTeX/ReX](https://github.com/ReTeX/ReX), created to make the library compile on modern Rust (1.80+).**

## Disclaimer

**This fork was produced with the assistance of an LLM (large language model).** The changes are mechanical in nature - replacing the broken `static_map` proc-macro dependency with `std::sync::LazyLock<HashMap>` - and have not been extensively hand-reviewed or tested beyond compilation and basic functionality. Use at your own risk.

This is not an official release. The original authors ([Christopher Breeden](https://github.com/cbreeden), [Sebastian Köln](https://github.com/kli6891)) are not affiliated with this fork.

## What changed

The upstream ReX depends on `static_map` / `static_map_macros` (v0.2.0-beta), which uses ancient proc-macro internals (`syn 0.11`, `quote 0.3`) that panic on Rust compilers newer than ~2020. The upstream project appears to be unmaintained (last commit years ago, [open issue about project status](https://github.com/ReTeX/ReX/issues/38)).

This fork:
- **Removes** the `static_map` / `static_map_macros` dependencies entirely
- **Replaces** all `static_map::Map<K, V>` statics with `LazyLock<HashMap<K, V>>` (stable since Rust 1.80)
- **Updates** the Rust edition from 2018 to 2021
- **Updates** `log` from 0.3 to 0.4, `env_logger` from 0.7 to 0.10
- **Removes** the `bincode` / `serde_yaml` dev-dependencies (outdated versions)
- Makes **no changes** to the parsing, layout, or rendering logic

The API is unchanged. `HashMap::get()` has the same signature as `static_map::Map::get()`, and `LazyLock` auto-derefs, so all downstream call sites work without modification.

## Original README

See the [upstream repository](https://github.com/ReTeX/ReX) for documentation, samples, and usage instructions.

## License

Same as the original: dual-licensed under MIT and Apache-2.0, with portions covered by various BSD-like licenses. See the upstream repo for full license texts.
