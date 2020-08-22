
# Pico-Lang

![Rust](https://github.com/c0d3x42/pico-rs/workflows/Rust/badge.svg?branch=master)

A minimal and safe programming language expressed in `JSON`



# Install

```bash
cargo install pico-lang
```


# build

**enable nats**
```bash
cargo run --features srv_nats
```

# lookups

lookup tables should be scoped to the rule file inclusion hierarchy.
each table is available to RuleFile that defined it, **and** files included below it
the lowest level included RuleFile has access to all lookups above it.
the root level RuleFile **onl** has access to the lookups defined in it


# warp submit

```bash
curl -v -X POST localhost:8000/submit -d '{"xp": "x1xxx", "y": "y2"}' -H 'Content-Type: application/json'
```

```bash
curl -v -X POST localhost:8000/submit -d '{"xp": "x1xxx", "y": "y2", "json": {"ja": "rules}}' -H 'Content-Type: application/json'
```

# benchmark

```bash
wrk -t 5 -c 40 -s bench/sub.lua http://localhost:8000/submit
```
