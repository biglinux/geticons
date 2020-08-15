all: geticons geticons.1

.PHONY: all

geticons: src/main.rs
	cargo build --release
	cp ./target/release/geticons .

geticons.1: docs/geticons.1.scd
	scdoc < docs/geticons.1.scd > geticons.1

clean:
	rm -r geticons geticons.1
