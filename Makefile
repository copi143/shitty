# hello

SHITTY_LIB_DIR := $(realpath ./lib)

.PHONY: help help/% init all clean lib

MAKEFILE := $(lastword $(MAKEFILE_LIST))

# 获取帮助信息
help:
	@scripts/help.sh "$(MAKEFILE)" || true

# 获取特定目标的帮助信息
help/%:
	@scripts/help.sh "$(MAKEFILE)" "$*" || true

init:
	mkdir -p ./local
	mkdir -p ./bin
	mkdir -p ./lib
	cargo fetch

all:

clean:
	rm -r ./bin || true
	rm -r ./lib || true
	cargo clean

# 构建 C 库并复制到 lib 目录
lib:
	@echo "Building C library..."
	cargo build -Z build-std=core,alloc --target x86_64-unknown-none --no-default-features --features ffi-default,font-unifont --release
	cp ./target/x86_64-unknown-none/release/libshitty.a ./lib/libshitty.a

ffi-examples: lib
	@echo "Building ffi examples..."
	$(MAKE) -C ./examples/c-x11 SHITTY_LIB_DIR=$(SHITTY_LIB_DIR)
	$(MAKE) -C ./examples/c-wayland SHITTY_LIB_DIR=$(SHITTY_LIB_DIR)

example:
	cargo build --features=bin,font-unifont

example-release:
	cargo build --features=bin,font-abglyph --release

run-example: example
	cargo run --features=bin,font-unifont -- bash -c local/run.sh

run-example-release: example-release
	cargo run --features=bin,font-abglyph --release -- bash -c local/run.sh

profile-example: example
	samply record target/debug/shitty local/run.sh

profile-example-release: example-release
	samply record target/release/shitty local/run.sh

build-test: # Maybe you need `rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu` to `build-std`.
	cargo build -Z build-std=core,alloc --target x86_64-unknown-none --no-default-features --features ffi-default,font-unifont
	cargo build -Z build-std=core,alloc --target x86_64-unknown-none --no-default-features --features ffi-default,font-truetype
	cargo build -Z build-std=core,alloc --target x86_64-unknown-none --no-default-features --features ffi-default,native,font-unifont
	cargo build -Z build-std=core,alloc --target x86_64-unknown-none --no-default-features --features ffi-default,native,font-truetype
	cargo build -Z build-std=core,alloc --target wasm32-unknown-unknown --no-default-features --features wasm-default,font-unifont
	cargo build -Z build-std=core,alloc --target wasm32-unknown-unknown --no-default-features --features wasm-default,font-truetype
	cargo build --features=bin,font-unifont
	cargo build --features=bin,font-truetype

ci-init:
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
	rustup target add x86_64-unknown-linux-gnu
	rustup target add x86_64-pc-windows-gnu
	sudo apt install -y gcc-mingw-w64-x86-64

ci-test: build-test cross-linux-test cross-windows-test
	cargo test

wasm-debug:
	wasm-pack build --target web --debug --no-default-features --features wasm-default,font-unifont
	rm -r ./www/pkg || true
	mv ./pkg ./www/pkg

wasm-release:
	wasm-pack build --target web --release --no-default-features --features wasm-default,font-unifont
	rm -r ./www/pkg || true
	mv ./pkg ./www/pkg

gen-icon:
	inkscape --export-type=png --export-filename=shitty.png --export-width=128 --export-height=128 docs/assets/shitty.svg
	magick shitty.png -depth 8 rgba:shitty.rgba
	mv shitty.rgba examples/shitty.rgba
	rm shitty.png

build-linux:
	cargo build --features=bin,font-unifont --release
	cp ./target/release/shitty ./bin/terminal-unifont
	cargo build --features=bin,font-truetype --release
	cp ./target/release/shitty ./bin/terminal-truetype

cross-linux-test:
	cargo build --target x86_64-unknown-linux-gnu --features=bin,font-unifont
	cargo build --target x86_64-unknown-linux-gnu --features=bin,font-truetype

cross-windows-test:
	cargo build --target x86_64-pc-windows-gnu --features=bin,font-unifont
	cargo build --target x86_64-pc-windows-gnu --features=bin,font-truetype

cross-linux:
	mkdir -p ./bin/cross-linux
	cargo build --target x86_64-unknown-linux-gnu --features=bin,font-unifont --release
	cp ./target/x86_64-unknown-linux-gnu/release/shitty ./bin/cross-linux/terminal-unifont
	cargo build --target x86_64-unknown-linux-gnu --features=bin,font-truetype --release
	cp ./target/x86_64-unknown-linux-gnu/release/shitty ./bin/cross-linux/terminal-truetype

cross-windows:
	mkdir -p ./bin/cross-windows
	cargo build --target x86_64-pc-windows-gnu --features=bin,font-unifont --release
	cp ./target/x86_64-pc-windows-gnu/release/shitty.exe ./bin/cross-windows/terminal-unifont.exe
	cargo build --target x86_64-pc-windows-gnu --features=bin,font-truetype --release
	cp ./target/x86_64-pc-windows-gnu/release/shitty.exe ./bin/cross-windows/terminal-truetype.exe

# 执行模糊测试
fuzzing:
	make -C tests/fuzzing

web: wasm-release
	sleep 1 && xdg-open http://127.0.0.1:8000/ &
	scripts/web.py
