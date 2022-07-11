all:
	cargo build --release

test:
	cargo run --release -- --supplementary sel test/test.sam -o target.sam

clean:
	cargo clean

.PHONY: all test clean
