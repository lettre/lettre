all: depends _book

depends:
	cargo install --force mdbook --vers "^0.2"
	cargo install --force mdbook-linkcheck --vers "^0.2"

serve:
	mdbook serve

_book:
	mdbook build

clean:
	rm -rf _book/

.PHONY: _book