# Python bindings

## Generate bindings

From repo root:

```bash
bin/generate-bindings.sh python
```

Generated files are written to:

- `bindings/python/src/json_joy_rs/generated`

## Package layout

- `src/json_joy_rs`: Python package namespace.
- `src/json_joy_rs/generated`: UniFFI-generated Python API and loader glue.

## Build wheel (placeholder)

```bash
cd bindings/python
python -m build
```
