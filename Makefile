RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTPKG ?= rustpkg
RUSTFLAGS ?= -O

#RUST_REPOSITORY ?= ../rust
#RUST_CTAGS ?= $(RUST_REPOSITORY)/src/etc/ctags.rust
VERSION=0.1-pre


xml_parser_so=build/libxml-rust-3c2055d-0.1.so
xml_parser_files=\
		$(wildcard src/xml/*.rs)

parser: $(xml_parser_so)

$(xml_parser_so): $(xml_parser_files)
		mkdir -p build/
		$(RUSTC) $(RUSTFLAGS) src/xml/lib.rs --out-dir=build

build/%:: src/%/main.rs $(libxml_so)
		mkdir -p "$(dir $@)"
		$(RUSTC) $(RUSTFLAGS) $< -o $@ -L build/


all: parser
#build/%:: src/%/main.rs $(libxml_so)
#        mkdir -p "$(dir $@)"
#        $(RUSTC) $(RUSTFLAGS) $< -o $@ -L build/

#examples: $(patsubst src/examples/%/main.rs,build/examples/%,$(wildcard src/examples/*/main.rs)) \
#                 $(patsubst src/examples/%/main.rs,build/examples/%,$(wildcard src/examples/*/*/main.rs))

docs: doc/http/index.html

doc/http/index.html: $(xml_parser_files)
		$(RUSTDOC) src/xml/lib.rs

build/tests: $(xml_parser_files)
		$(RUSTC) $(RUSTFLAGS) --test -o build/tests src/test/test.rs -L build/
		$(RUSTC) $(RUSTFLAGS) --test -o build/inter_test src/xml/lib.rs

check: clean all build/tests
		build/tests --test
		build/inter_test --test

clean:
		rm -rf build/

clean-docs:
		rm -rf doc/


.PHONY: all parser clean check clean-docs
