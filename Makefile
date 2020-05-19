
.PHONY: build-release release-linux-musl

build-release:
	cargo build --release

release-mac: build-release
	strip target/release/panopticon-tui
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/panopticon-tui-mac.tar.gz ./panopticon-tui

release-win: build-release
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/panopticon-tui-win.tar.gz ./panopticon-tui.exe

release-linux-musl:
	cargo build --release --target=x86_64-unknown-linux-musl
	strip target/x86_64-unknown-linux-musl/release/panopticon-tui
	mkdir -p release
	tar -C ./target/x86_64-unknown-linux-musl/release/ -czvf ./release/panopticon-tui-linux-musl.tar.gz ./panopticon-tui
