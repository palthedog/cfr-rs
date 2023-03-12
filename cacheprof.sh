#!/bin/bash

# callgrind doesn't support avx512 and similars?
RUSTFLAGS="-C target-cpu=x86-64" cargo build --workspace --release

OUTFILE=cachegrind.tmp.out
valgrind --tool=cachegrind --branch-sim=yes --cachegrind-out-file=$OUTFILE ./target/release/cfr --game leduc --duration 10s mccfr-external-sampling &&
    cg_annotate $OUTFILE
