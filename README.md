# Counterfactual Regret Minimization playground written in Rust

## Reference: An Introduction to Counterfactual Regret Minimization
http://modelai.gettysburg.edu/2013/cfr/cfr.pdf

# Results
## Leduc Poker
```
$ cargo run -p cfr --release -- --game leduc cfr --iterations 1000000 -l logs/leduc/cfr.csv
    Compiling cfr v0.1.0 (/home/niwasaki/work/cfr-rs/cfr)
    Finished release [optimized + debuginfo] target(s) in 1.18s
    Running `target/release/cfr -g leduc -i 1000000 -l logs/leduc/tmp.csv`
[2022-12-19T12:09:15Z INFO  cfr::eval] Calculating best response for player 0
[2022-12-19T12:09:15Z INFO  cfr::eval] Calculating best response for player 1
[2022-12-19T12:09:15Z INFO  cfr::eval] util_0(br0): -0.0785755900494477, util_1(br1): 0.09176988168639028
[2022-12-19T12:09:15Z INFO  cfr::eval] util_1(s0, s_br1): 0.09176988168639028 util_0(s_br0, s1): -0.0785755900494477
... snipped ...
$ cargo run -p explot
   Compiling explot v0.1.0 (/home/niwasaki/work/cfr-rs/explot)
    Finished dev [optimized + debuginfo] target(s) in 0.42s
     Running `target/debug/explot`
[2022-12-19T12:09:32Z INFO  explot] plotting: cfr.csv
[2022-12-19T12:09:32Z INFO  explot] graphs/leduc.svg created
```
![leduc_exploitability](./graphs/leduc.svg)
