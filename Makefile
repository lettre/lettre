RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTFLAGS ?= -g
BUILDDIR ?= build
INSTALLDIR ?= /usr/local/lib
DOCDIR ?= doc

SMTP_LIB := src/smtp/lib.rs

libsmtp=$(shell $(RUSTC) --crate-file-name $(SMTP_LIB))

smtp_files=\
	$(wildcard src/smtp/*.rs) \
	$(wildcard src/smtp/client/*.rs)

example_files=\
	$(wildcard src/examples/*.rs)

smtp: $(libsmtp)

$(libsmtp): $(smtp_files)
	mkdir -p $(BUILDDIR)
	$(RUSTC) $(RUSTFLAGS) $(SMTP_LIB) --out-dir=$(BUILDDIR)

all: smtp examples doc

doc: $(smtp_files)
	$(RUSTDOC) $(SMTP_LIB)

examples: smtp $(example_files)
	$(RUSTC) $(RUSTFLAGS) -L $(BUILDDIR)/ src/examples/client.rs --out-dir=$(BUILDDIR)

$(BUILDDIR)/tests: $(smtp_files)
	mkdir -p $(BUILDDIR)/tests
	$(RUSTC) --test $(SMTP_LIB) --out-dir=$(BUILDDIR)/tests

check: all $(BUILDDIR)/tests
	$(BUILDDIR)/tests/smtp --test

install: $(libsmtp_so)
	install $(libsmtp_so) $(INSTALLDIR)

clean:
	rm -rf $(BUILDDIR)
	rm -rf $(DOCDIR)

.PHONY: all smtp examples docs clean check
