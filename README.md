# Substrate with ERC20

This is the [substrate template node](https://github.com/substrate-developer-hub/substrate-node-template)
with a [pallet](https://docs.substrate.io/learn/runtime-development/#frame) that implements [ERC20](https://eips.ethereum.org/EIPS/eip-20)-_like_ functionality 

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
- [X] Remove u32 -> T::Balance conversions
- [ ] Weights, benchmarks
- [ ] Refactor of code structure - implement `tokens::funginle`, move common functionality in separate functions, change how storage gets modified
- [ ] Unit tests (integration testing of substrate is whole another story)
- [x] Replace some direct storage access with `storage-getters`

### Build

Use the following command to build the node without launching it:

```sh
cargo build --release
```

### Run

Most primitive way to test functionality of a pallet provided in this repo is to run two nodes which will _simulate_ basic private/solo network

1. ```shell
   ./target/release/node-template \
   --base-path /tmp/node01 \
   --rpc-external \
   --alice \
   --rpc-port 9945 \
   --port 30333 \
   --validator \
   --rpc-methods Unsafe \
   --name Node01
   ```

2. ```shell
   ./target/release/node-template \
   --base-path /tmp/node02 \
   --bob \
   --port 30334 \
   --rpc-port 9946 \
   --validator \
   --rpc-methods Unsafe \
   --name Node02 \
   ```

Node discovery happens _automatically_ in **local** (rfc1918) networks so we **don't** need to specify bootnode.

Then you can connect to one of those with [polkadot.js.org/apps](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9945#/explorer)

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

