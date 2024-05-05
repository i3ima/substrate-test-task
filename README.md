# Substrate with ERC20

This is the [substrate template node](https://github.com/substrate-developer-hub/substrate-node-template)
with a [pallet](https://docs.substrate.io/learn/runtime-development/#frame) that implements [ERC20](https://eips.ethereum.org/EIPS/eip-20)-like functionality 

## Notes:
- Pallet code is _very_ dirty. Manual storage set & mutate _should_ be replaces with usage of `frame_support::traits::tokens::fungible` 
  which provides trait to manage _fungible tokens_ as as stated in 
  [docs](https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_tokens/index.html#fungible-token-traits-in-frame).
  DRY **needs** to be applied to many places with code duplication, _mainly_ transfer functions.
- Some weird things can _possibly_ happen with `u32 -> Balance (u128)` conversions you can see throughout code. Although `u32` is a subset of `u128`
  and arithmetic operations performed by `sp_arithmetic` with `saturating_*`/`checked_*` whenever possible I'd still check for 
  any possible corner cases
- Pallet is implemented as _instantiable_. Which means that by providing different types to `Config<I>` we can have multiple instances of it 
  in one runtime

## TODO: 
- Weights, benchmarks
- Unit tests (integration testing of substrate is whole another story)
- Refactor of code structure - implement `tokens::funginle`, move common functionality in separate functions, change how storage gets modified

### Build

Use the following command to build the node without launching it:

```sh
cargo build --release
```

### Run



### Docs

After you build the project, you can use the following command to explore its
parameters and subcommands:

```sh
./target/release/node-template -h
```

You can generate and view the [Rust
Docs](https://doc.rust-lang.org/cargo/commands/cargo-doc.html) for this template
with this command:

```sh
cargo +nightly doc --open
```

