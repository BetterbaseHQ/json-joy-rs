.PHONY: check fmt test bindings-python

check:
	mise x -- cargo check

fmt:
	mise x -- cargo fmt --all

test:
	mise x -- cargo test --workspace

bindings-python:
	bin/generate-bindings.sh python
