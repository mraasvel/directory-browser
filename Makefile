all: run

run: build
	cargo run 2> browser.log
build:
	cargo build

.PHONY: all run build
