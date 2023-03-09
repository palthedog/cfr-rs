#!/bin/bash

# callgrind doesn't support avx512 and similars?
RUSTFLAGS="-C target-cpu=x86-64" cargo build --workspace --release

OUTFILE=callgrind.tmp.out
valgrind --tool=callgrind --callgrind-out-file=$OUTFILE --cache-sim=yes ./target/release/cfr --game leduc --duration 10s mccfr-external-sampling &&
    kcachegrind $OUTFILE
