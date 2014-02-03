RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTPKG ?= rustpkg
RUSTFLAGS ?= -O -Z debug-info
VERSION=0.1-pre

libsmtp_so=build/libsmtp-4c61a8ad-0.1-pre.so

smtp_files=\
	$(wildcard src/smtp/*.rs) \
	$(wildcard src/smtp/client/*.rs)

example_files=\
	src/examples/client.rs

smtp: $(libsmtp_so)

$(libsmtp_so): $(smtp_files)
	mkdir -p build/
	$(RUSTC) $(RUSTFLAGS) src/smtp/lib.rs --out-dir=build

all: smtp examples docs

docs: doc/smtp/index.html

doc/smtp/index.html: $(smtp_files)
	$(RUSTDOC) src/smtp/lib.rs

examples: smtp $(example_files)
	$(RUSTC) $(RUSTFLAGS) -L build/ src/examples/client.rs -o build/client

build/tests: $(smtp_files)
	$(RUSTC) --test -o build/tests src/smtp/lib.rs

check: all build/tests
	build/tests --test

clean:
	rm -rf build/
	rm -rf doc/

.PHONY: all smtp examples docs clean check tests
