# Octopus Appchain Pallet

This is a support component of [Octopus Network](https://oct.network/).

## Purpose

With this pallet, a chain built from substrate can join octopus network as an appchain.

An appchain can rent security from motherchain on demand.

## Dependencies

### Traits

This pallet depend on [`CreateSignedTransaction`](https://docs.rs/frame-system/3.0.0/frame_system/offchain/trait.CreateSignedTransaction.html).

### Pallets

This pallet depend on [`pallet_session`](https://docs.rs/pallet-session/3.0.0/pallet_session/).

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
[dependencies]
pallet-session = { default-features = false, version = '3.0.0' }
pallet-octopus-appchain = { default-features = false, git = 'https://github.com/octopus-network/pallet-octopus-appchain.git', branch = 'cargo-fix' }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-session/std',
    'pallet-octopus-appchain/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
parameter_types! {
	pub const AppchainId: pallet_octopus_appchain::ChainId = 0;
	pub const Motherchain: pallet_octopus_appchain::MotherchainType = pallet_octopus_appchain::MotherchainType::NEAR;
	pub const GracePeriod: u32 = 5;
	pub const UnsignedPriority: u64 = 1 << 20;
}

impl pallet_octopus_appchain::Config for Runtime {
	type Event = Event;
	type AuthorityId = pallet_octopus_appchain::crypto::OctopusAuthId;
	type Call = Call;
	type AppchainId = AppchainId;
	type Motherchain = Motherchain;
	const RELAY_CONTRACT_NAME: &'static [u8] = b"dev-1616239154529-4812993";
	type GracePeriod = GracePeriod;
	type UnsignedPriority = UnsignedPriority;
}
```

and include it in your `construct_runtime!` macro:

```rust
OctopusAppchain: pallet_octopus_appchain::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},
```

### Genesis Configuration

See [this commit of Barnacle](https://github.com/octopus-network/barnacle/commit/6bf1c8f0479887af17535024160b4ad55482dc31) for genesis configuration and other settings.

We will explain these configurations in detail later.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
