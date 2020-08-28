
# Pico-Lang

![Rust](https://github.com/c0d3x42/pico-rs/workflows/Rust/badge.svg?branch=master)

A minimal and safe programming language expressed in `JSON`



# Install

```bash
cargo install pico-lang
```


# build

```bash
cargo build
```

**or** just run it

```bash
cargo run
```

**enable nats** (incomplete)
```bash
cargo run --features srv_nats
```

# PicoRules

`JSON` formmated file that encapsulates your logic

see [simple.json](/simple.json) example rule file

start server with
```bash
cargo run -- --rules simple.json
```

submit rule execution

```bash
curl -X POST localhost:8000/submit -d '{"nochicken": 1}' -H 'Content-Type: application/json'
```
returns:
```
{"namespaced":{},"input":{"nochicken":1},"locals":{"enochicken":"must be no hens"}}
```

and with a chicken:
```bash
curl -X POST localhost:8000/submit -d '{"chicken": 1}' -H 'Content-Type: application/json'
```
returns:
```
{"locals":{"egg":"must have been layed"},"namespaced":{},"input":{"chicken":1}}
```

# warp submit

```bash
curl -v -X POST localhost:8000/submit -d '{"xp": "x1xxx", "y": "y2"}' -H 'Content-Type: application/json'
```

```bash
curl -v -X POST localhost:8000/submit -d '{"xp": "x1xxx", "y": "y2", "json": {"ja": "rules"}}' -H 'Content-Type: application/json'
```

# benchmark

```bash
wrk -t 5 -c 40 -s bench/sub.lua http://localhost:8000/submit
```
