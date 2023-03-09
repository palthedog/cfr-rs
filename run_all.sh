#!/bin/bash

DURATION=${1:-5s}
shift
GAMES=${@:-kuhn leduc dudo}

echo duration $DURATION
echo games $GAMES

for game in $GAMES; do
    for solver in cfr mccfr-external-sampling; do
        cargo run --release -- --game $game --duration $DURATION --log-path logs/${game}/${solver}.csv $solver
    done
done

cargo run -p explot
