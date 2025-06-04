

all:
	maturin develop -m oc_user/Cargo.toml
	
check:
	cargo check -p main

run:
	cargo run -p main

clean:
	cargo clean
