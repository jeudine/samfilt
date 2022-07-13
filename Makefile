all:
	cargo build --release

test:
	cargo run --release -- --supplementary sel --qname_output name.txt  test/test.sam -o target0.sam
	cargo run --release -- --qname_input name.txt test/test.sam -o target1.sam

clean:
	cargo clean

.PHONY: all test clean
