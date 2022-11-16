# An Introduction to Counterfactual Regret Minimization
http://modelai.gettysburg.edu/2013/cfr/cfr.pdf

## Rock Paper Scissors
```
$ cargo run --release -p rps
...
[2022-11-15T14:07:27Z INFO  rps] Player: Regret[683, -342, -29, ]
    Strategy[683, 0, 0, ]
    Avg-Strategy[0.33529, 0.33250, 0.33221, ]
```

## Colonel Blotto
```
$ cargo run --release -p blotto
...
[2022-11-15T14:06:20Z INFO  blotto] Avg-Strategy [
      Strategy    Probability
      (5,0,0,)    0.00000
      (4,1,0,)    0.00000
      (3,2,0,)    0.11356
      (2,3,0,)    0.10573
      (1,4,0,)    0.00000
      (0,5,0,)    0.00000
      (4,0,1,)    0.00000
      (3,1,1,)    0.10943
      (2,2,1,)    0.00000
      (1,3,1,)    0.11118
      (0,4,1,)    0.00000
      (3,0,2,)    0.11865
      (2,1,2,)    0.00000
      (1,2,2,)    0.00000
      (0,3,2,)    0.10141
      (2,0,3,)    0.11519
      (1,1,3,)    0.10873
      (0,2,3,)    0.11611
      (1,0,4,)    0.00000
      (0,1,4,)    0.00000
      (0,0,5,)    0.00000
    ]
```

## Kuhn Poker
```
$ cargo run --release -p kuhn
    Finished release [optimized + debuginfo] target(s) in 0.01s
     Running `/home/niwasaki/work/cfr-rs/target/release/kuhn`
[2022-11-16T19:32:05Z INFO  kuhn] Training has finished
[2022-11-16T19:32:05Z INFO  kuhn] Average game value: -0.05569656776233496
[2022-11-16T19:32:05Z INFO  kuhn] Nodes [
[2022-11-16T19:32:05Z INFO  kuhn]     Node(0 Jack , [None       ,None       ]): Pass: 0.8865, Bet: 0.1135
[2022-11-16T19:32:05Z INFO  kuhn]     Node(0 Queen, [None       ,None       ]): Pass: 1.0000, Bet: 0.0000
[2022-11-16T19:32:05Z INFO  kuhn]     Node(0 King , [None       ,None       ]): Pass: 0.6571, Bet: 0.3429
[2022-11-16T19:32:05Z INFO  kuhn]     Node(1 Jack , [Some(Pass) ,None       ]): Pass: 0.6663, Bet: 0.3337
[2022-11-16T19:32:05Z INFO  kuhn]     Node(1 Queen, [Some(Pass) ,None       ]): Pass: 1.0000, Bet: 0.0000
[2022-11-16T19:32:05Z INFO  kuhn]     Node(1 King , [Some(Pass) ,None       ]): Pass: 0.0000, Bet: 1.0000
[2022-11-16T19:32:05Z INFO  kuhn]     Node(0 Jack , [Some(Pass) ,Some(Bet)  ]): Pass: 1.0000, Bet: 0.0000
[2022-11-16T19:32:05Z INFO  kuhn]     Node(0 Queen, [Some(Pass) ,Some(Bet)  ]): Pass: 0.5528, Bet: 0.4472
[2022-11-16T19:32:05Z INFO  kuhn]     Node(0 King , [Some(Pass) ,Some(Bet)  ]): Pass: 0.0000, Bet: 1.0000
[2022-11-16T19:32:05Z INFO  kuhn]     Node(1 Jack , [Some(Bet)  ,None       ]): Pass: 1.0000, Bet: 0.0000
[2022-11-16T19:32:05Z INFO  kuhn]     Node(1 Queen, [Some(Bet)  ,None       ]): Pass: 0.6668, Bet: 0.3332
[2022-11-16T19:32:05Z INFO  kuhn]     Node(1 King , [Some(Bet)  ,None       ]): Pass: 0.0000, Bet: 1.0000
[2022-11-16T19:32:05Z INFO  kuhn] ]
```
