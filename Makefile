TARGET_DIR := ./target/release
BIN_NAME := histdiff
INSTALL_DIR := .

all: build install

build:
	cargo build --release

install: build
	ln -s $(TARGET_DIR)/$(BIN_NAME) ./$(BIN_NAME)

clean:
	cargo clean
	rm ./$(BIN_NAME)

.PHONY: all build install clean
