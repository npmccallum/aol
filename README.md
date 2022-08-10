# Append-Only Log Benchmarks

The purpose of this repo is to do an approximation benchmark of the "linear"
versus "trillian" data structures. The main difference between the two is that
the "trillian" structure has only a single signature at the head and the
"linear" data structure signs each message. So we approximate this by
constructing a linear version of the log and then verifying it either with
signatures or without. In both cases, we hash the messages and validate the
message hashes. Therefore, this is a rough approximation of the differing data
structures.

# Results on an M1 MacBook Pro

```
$ cargo run --release 100000
    Finished release [optimized] target(s) in 0.05s
     Running `target/release/aol 100000`
entries: 100000
time: 16.195103833s
size: 16.599971999999998 MB
sig: true
time: 40.84065975s
byte: Some(29)

$ cargo run --release 100000 nosig
    Finished release [optimized] target(s) in 0.09s
     Running `target/release/aol 100000 nosig`
entries: 100000
time: 16.06762675s
size: 16.599971999999998 MB
sig: false
time: 123.180291ms
byte: Some(124)
```
