all:
	cargo build --release

test:
	cargo run --release -- --supplementary sel --smaller_len 16328  test/test.sam -o target.sam
	cargo run --release -- -h

clean:
	cargo clean

.PHONY: all test clean
