TARGET_DIR := ./target/release
BIN_NAME := histdiff signals_formatter
INSTALL_DIR := .

all: build install

build:
	cargo build --release

install: build
	ln -s $(TARGET_DIR)/histdiff ./histdiff_bin
	ln -s $(TARGET_DIR)/signals_formatter ./signals_formatter_bin
	# for bin in $(BIN_NAME); do \
	# 	ln -s $(TARGET_DIR)/$$bin ./$$bin; \
	# done

clean:
	cargo clean
	rm ./histdiff_bin
	rm ./signals_formatter_bin
	# for bin in $(BIN_NAME); do \
	# 	rm ./$$bin; \
	# done

.PHONY: all build install clean
