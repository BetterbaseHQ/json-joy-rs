# Python bindings

Upstream Credit: These bindings target the Rust port of upstream
`json-joy` by streamich.
- Upstream repository: <https://github.com/streamich/json-joy>
- Upstream docs: <https://jsonjoy.com/libs/json-joy-js>

This directory contains Python packaging for `json-joy-rs` generated via
UniFFI.

## Generate bindings

From repository root:

```bash
bin/generate-bindings.sh python
```

Generated files are written to:

- `bindings/python/src/json_joy_rs/generated`

## Package layout

- `src/json_joy_rs`: Python package namespace
- `src/json_joy_rs/generated`: UniFFI-generated API bindings and loader glue

## Build

```bash
cd bindings/python
python -m build
```
