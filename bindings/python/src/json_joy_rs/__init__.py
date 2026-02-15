"""Python package namespace for json-joy-rs bindings.

Generated UniFFI Python modules and shared-library loader stubs are expected in
`json_joy_rs/generated/`.
"""

from pathlib import Path

_generated_dir = Path(__file__).with_name("generated")


def generated_dir() -> Path:
    """Return the directory where generated UniFFI Python bindings live."""
    return _generated_dir
