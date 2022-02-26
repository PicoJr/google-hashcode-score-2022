[![GitHub license](https://img.shields.io/github/license/PicoJr/google-hashcode-score-2022)](https://github.com/PicoJr/google-hashcode-score-2022/blob/master/LICENSE)

# Google Hashcode 2022 Score Calculator

Computes Google Hashcode 2022 Qualification Round score.

It gives the same results as Google for our submissions.

It gives the same result for the example file as well.

**Caution**: It may not be robust against incorrect submission file.

## Quickstart

### Install Rust Toolchain using rustup

cf https://www.rust-lang.org/learn/get-started

#### On Unix

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### On Windows

refer to https://rustup.rs (download and run `rustup-init.exe`)

### Compile and run

```
cargo run --release res/*.txt -o out/*.out
```

output:

```
out/a_an_example.in.txt.out score: 33
out/b_better_start_small.in.txt.out score: 275,518
out/c_collaboration.in.txt.out score: 167,256
out/d_dense_schedule.in.txt.out score: 251,751
out/e_exceptional_skills.in.txt.out score: 1,594,950
out/f_find_great_mentors.in.txt.out score: 456,349
total score: 2,745,857
```

> Note: only `out/a_an_example.in.txt.out` is provided for now as the extended round is ongoing.

### Performance

cpu: `AMD Ryzen 7 3700X`

```
❯ hyperfine "cargo run --release res/*.txt -o out/*.out"
Benchmark 1: cargo run --release res/*.txt -o out/*.out
  Time (mean ± σ):     213.5 ms ±   6.1 ms    [User: 189.1 ms, System: 23.7 ms]
  Range (min … max):   205.9 ms … 226.7 ms    13 runs
```

with checks disabled:

```
❯ hyperfine "cargo run --release res/*.txt -o out/*.out --disable-checks"
Benchmark 1: cargo run --release res/*.txt -o out/*.out --disable-checks
  Time (mean ± σ):     189.5 ms ±   2.4 ms    [User: 166.8 ms, System: 22.0 ms]
  Range (min … max):   186.2 ms … 194.5 ms    15 runs
```

### Enable debug logs

```
RUST_LOG=debug cargo run res/a_an_example.in.txt -o out/a_an_example.in.txt.out
```

### License

Dual-licensed under MIT or the Apache License V2.0.