
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