# Hermes (Conrad)

My playground for a basic payments engine.

## Todo

- [x] Split code into separate modules for better separation
- [ ] Add more documentation
- [ ] Add more tests
- [ ] Add benchmarks

## Tests

Right now, only the Engine struct has test. They are located in `src/engine.rs`.

## Assumptions

- For now, only handle dispute for `deposit` events

## Ideas

I would love to make a version using CQRS and event sourcing ([cqrs-es](https://lib.rs/crates/cqrs-es)).
