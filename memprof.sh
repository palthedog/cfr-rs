#!/bin/bash

# callgrind doesn't support avx512 and similars?
RUSTFLAGS="-C target-cpu=x86-64" cargo build --workspace --release

OUTFILE=massif.tmp.out
valgrind --tool=massif --massif-out-file=$OUTFILE ./target/release/cfr --game leduc --duration 10s mccfr-external-sampling &&
massif-visualizer $OUTFILE
