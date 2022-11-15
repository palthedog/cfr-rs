# An Introduction to Counterfactual Regret Minimization
http://modelai.gettysburg.edu/2013/cfr/cfr.pdf

## Rock Paper Scissors
```
> cargo run --release -p rps
...
[2022-11-15T14:07:27Z INFO  rps] Player: Regret[683, -342, -29, ]
    Strategy[683, 0, 0, ]
    Avg-Strategy[0.33529, 0.33250, 0.33221, ]
```

## Colonel Blotto
```
> cargo run --release -p blotto
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
