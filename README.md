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

### Compile and start server

```
❯ cargo run --release res/*.txt
```

output:

```
Finished release [optimized + debuginfo] target(s) in 0.04s
Running `target/release/google-hashcode-score-2022 res/a_an_example.in.txt res/b_better_start_small.in.txt res/c_collaboration.in.txt res/d_dense_schedule.in.txt res/e_exceptional_skills.in.txt res/f_find_great_mentors.in.txt`
Listening on http://([127, 0, 0, 1], 3000)
```

### Submit scores

Using [jq](https://github.com/stedolan/jq) to parse JSON response.

```
curl --silent --request POST --data-binary @out/a_an_example.in.txt.out http://localhost:3000/score/0 | jq .score
```

> `33`

```
curl --silent --request POST --data-binary @out/b_better_start_small.in.txt.out http://localhost:3000/score/1 | jq .score
```

> `275518`

```
curl --silent --request POST --data-binary @out/c_collaboration.in.txt.out http://localhost:3000/score/2 | jq .score
```

> `167256`

```
curl --silent --request POST --data-binary @out/d_dense_schedule.in.txt.out http://localhost:3000/score/3 | jq .score
```

> `251751`

```
curl --silent --request POST --data-binary @out/e_exceptional_skills.in.txt.out http://localhost:3000/score/4 | jq .score
```

> `1594950`

```
curl --silent --request POST --data-binary @out/f_find_great_mentors.in.txt.out http://localhost:3000/score/5 | jq .score
```

> `456349`


## Performance

cpu: `AMD Ryzen 7 3700X`

`parallel` cf <https://www.gnu.org/software/parallel/>

### With checks enabled

start server: `cargo run --release res/*.txt`

```
❯ hyperfine "./parallel -a curls_benchmark.txt"
Benchmark 1: ./parallel -a curls_benchmark.txt
  Time (mean ± σ):     100.4 ms ±   5.8 ms    [User: 77.3 ms, System: 40.9 ms]
  Range (min … max):    93.7 ms … 110.2 ms    29 runs
```

### With checks disabled (with cache)

start server: `cargo run --release res/*.txt --disable-checks`

```
❯ hyperfine "./parallel -a curls_benchmark.txt"
Benchmark 1: ./parallel -a curls_benchmark.txt
  Time (mean ± σ):      92.3 ms ±   5.3 ms    [User: 77.2 ms, System: 40.3 ms]
  Range (min … max):    85.6 ms … 101.4 ms    29 runs
```

## Enable debug logs

```
RUST_LOG=debug cargo run --release res/*.txt
```

## License

Dual-licensed under MIT or the Apache License V2.0.
