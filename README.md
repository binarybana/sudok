## Sudok
Yes, yes, _another_ sudoku solver. But I had to do something while everyone around me solved them by hand...

(also, I avoided the temptation to look at [Norvig's solution](http://norvig.com/sudoku.html), instead I just used his storage format and examples)

...

Okay, after finishing, I looked at Norvig's and we both used a backtracking solver with constraints, though he had done his constraint updates point-wise rather than global, which gave a nice speedup to my rust version.

Oddly, this is still slower than Norvig's even after a little bit of effort optimizing:
```
$ RUSTFLAGS="-C target-cpu=native" cargo build --release
$ target/release/sudok top95.txt

Time for 95 puzzles: Duration { secs: 1, nanos: 523574332 }, time per puzzle: Duration { secs: 0, nanos: 16037623 }
```
vs:
```
$ python sudoku.py
All tests pass.
Solved 95 of 95 hard puzzles (avg 0.01 secs (117 Hz), max 0.04 secs).
```

So 10ms avg for Norvig and 16ms avg for mine (as of 9a4a370).

Optimizations made so far:
- Change to pointwise constraint update instead of global (following Norvig)
- Use BitSet instead of vector to store possible options (Norvig uses a string here)

Optimization ideas I still have:
- Possibly use 3D bit cube to represent board (with third dimension being the digits), plus augmented bit matrix for solved or not etc.
- Cache set of peers instead of calculating row-wise, col-wise, and block-wise peers (with overlaps) everytime. (Norvig does this already)
