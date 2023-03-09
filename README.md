# A Counterfactual Regret Minimization (CFR) playground written in Rust

## Reference: An Introduction to Counterfactual Regret Minimization
http://modelai.gettysburg.edu/2013/cfr/cfr.pdf

# Results
## Leduc Poker
```
$ cargo run --release -- --game leduc --duration 1h --log-path logs/leduc/cfr.csv cfr
... snipped ...
$ cargo run --release -- --game leduc --duration 1h --log-path logs/leduc/mccfr-external-sampling.csv mccfr-external-sampling
... snipped ...
$ cargo run -p explot
   Compiling explot v0.1.0 (/home/niwasaki/work/cfr-rs/explot)
    Finished dev [optimized + debuginfo] target(s) in 0.42s
     Running `target/debug/explot`
[2022-12-19T12:09:32Z INFO  explot] plotting: cfr.csv
[2022-12-19T12:09:32Z INFO  explot] graphs/leduc.svg created
```
![leduc_exploitability](./graphs/leduc.svg)
