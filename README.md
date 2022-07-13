# samfilt

A filtering tool for Sequence Alignment/Map files written in Rust.

## Installation

### From source
if you want to build samfilt from source, you need Rust. You can then use `cargo` to build everything:

```bash
cargo install samfilt
```

## Usage

```
Usage: samfilt [options] <SAM file>

Options:
    -h, --help          print this help menu
        --supplementary sel|del
                        reads with supplementary alignments
        --greater_len UINT
                        reads with a greater length than UINT [0]
        --smaller_len UINT
                        reads with a smaller length than UINT [4294967295]
        --qname_input FILE
                        alignment records with QNAME being equal to one of the
                        lines in FILE
        --qname_output FILE
                        output the QNAME field of all the records in the
                        filtered SAM file (one per line)
    -o FILE             output to FILE [stdout]
```
