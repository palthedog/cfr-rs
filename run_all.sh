#!/bin/bash

DURATION=${1:-5s}

echo duration $DURATION

for game in kuhn leduc dudo; do
    for solver in cfr mccfr-external-sampling; do
        cargo run --release -- --game $game --duration $DURATION --log-path logs/${game}/${solver}.csv $solver
    done
done

cargo run -p explot
