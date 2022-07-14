# Scoring

This project was made very quickly, so it's probably not your best place to learn from.

## Building for IRIS

IRIS laptops require an outdated version of libc.
To build for there laptops, use cross:

```
cargo install cross
```

Then you can build it like:

```
cross build --release
```
