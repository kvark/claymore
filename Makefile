.PHONY: all

all: lib

lib:
	cargo build

run: lib
	target/claymore
