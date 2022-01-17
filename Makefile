.PHONY: default wasm release wasm-release serve clean

default: wasm
	cargo build

wasm:
	cd frontend && wasm-pack build --target web --out-name package --dev

release: wasm-release
	cargo build --release

wasm-release:
	cd frontend && wasm-pack build --target web --out-name package

serve: default
	cargo run -p backend

clean:
	rm -r frontend/pkg
	cargo clean
