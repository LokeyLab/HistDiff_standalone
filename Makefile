TARGET_DIR := ./target/release
BIN_NAME := histdiff signals_formatter
INSTALL_DIR := .

all: build install

build:
	cargo build --release

install: build
	for bin in  $(BIN_NAME); do \
		ln -s $(TARGET_DIR)/$$bin ./$$bin; \
	done

clean:
	cargo clean
	for bin in $(BIN_NAME); do \
		rm ./$$bin; \
	done

.PHONY: all build install clean
