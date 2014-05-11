RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTC_FLAGS ?= -g

BIN_DIR = bin
DOC_DIR = doc
SRC_DIR = src
TARGET_DIR = target
EXAMPLES_DIR = examples
LIB = src/lib.rs

EXAMPLE_FILES := $(EXAMPLES_DIR)/*.rs
SOURCE_FILES := $(shell test -e src/ && find src -type f)

TARGET := $(shell rustc --version | awk "/host:/ { print \$$2 }")
TARGET_LIB_DIR := $(TARGET_DIR)/$(TARGET)/lib

RLIB_FILE := $(shell rustc --crate-type=rlib --crate-file-name "src/lib.rs" 2> /dev/null)
RLIB := $(TARGET_LIB_DIR)/$(RLIB_FILE)
DYLIB_FILE := $(shell rustc --crate-type=dylib --crate-file-name "src/lib.rs" 2> /dev/null)
DYLIB := $(TARGET_LIB_DIR)/$(DYLIB_FILE)

all: lib

lib: rlib dylib

rlib: $(RLIB)

$(RLIB): $(SOURCE_FILES) | $(LIB) $(TARGET_LIB_DIR)
	$(RUSTC) --target $(TARGET) $(RUSTC_FLAGS) --crate-type=rlib $(LIB) --out-dir $(TARGET_LIB_DIR)

dylib: $(DYLIB)

$(DYLIB): $(SOURCE_FILES) | $(LIB) $(TARGET_LIB_DIR)
	$(RUSTC) --target $(TARGET) $(RUSTC_FLAGS) --crate-type=dylib $(LIB) --out-dir $(TARGET_LIB_DIR)

$(TARGET_LIB_DIR):
	mkdir -p $(TARGET_LIB_DIR)

test: $(BIN_DIR)/test

$(BIN_DIR)/test: $(SOURCE_FILES) | rlib $(BIN_DIR)
	$(RUSTC) --target $(TARGET) $(RUSTC_FLAGS) --test $(LIB) -o $(BIN_DIR)/test -L $(TARGET_LIB_DIR)

doc: $(SOURCE_FILES)
	mkdir -p $(DOC_DIR)
	$(RUSTDOC) $(LIB) -L $(TARGET_LIB_DIR) -o $(DOC_DIR)

examples: $(EXAMPLE_FILES)

$(EXAMPLE_FILES): lib | $(BIN_DIR)
	$(RUSTC) --target $(TARGET) $(RUSTC_FLAGS) $@ -L $(TARGET_LIB_DIR) --out-dir $(BIN_DIR)

$(BIN_DIR):
	mkdir -p $(BIN_DIR)

check: test
	$(BIN_DIR)/test --test

clean:
	rm -rf $(TARGET_DIR)
	rm -rf $(DOC_DIR)
	rm -rf $(BIN_DIR)

.PHONY: all lib rlib dylib test doc examples clean check
