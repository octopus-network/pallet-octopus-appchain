# Octopus Appchain Pallet
[![crates.io](https://img.shields.io/crates/v/pallet-octopus-appchain.svg)](https://crates.io/crates/pallet-octopus-appchain)
[![Released API docs](https://docs.rs/pallet-octopus-appchain/badge.svg)](https://docs.rs/pallet-octopus-appchain)

This is a support component of [Octopus Network](https://oct.network/).

## Purpose

With this pallet, a chain built from substrate can join octopus network as an appchain.

An appchain can rent security from motherchain on demand.

## Appchain Guide

### Integration

#### Edit Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
[dependencies]
pallet-session = { default-features = false, version = '3.0.0' }
pallet-octopus-appchain = { default-features = false, git = 'https://github.com/octopus-network/pallet-octopus-appchain.git' }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-session/std',
    'pallet-octopus-appchain/std',
]
```

**Traits**

This pallet depend on [`CreateSignedTransaction`](https://docs.rs/frame-system/3.0.0/frame_system/offchain/trait.CreateSignedTransaction.html).

**Pallets**

This pallet depend on [pallet_session](https://docs.rs/pallet-session/3.0.0/pallet_session/).


#### Edit Runtime `lib.rs`

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

#### Genesis Configuration

See [this commit of Barnacle](https://github.com/octopus-network/barnacle/commit/5ebef3083ba8e233d35e9e8bb66c71eecca140e6) for genesis configuration and other settings.

We will explain these configurations in detail later.


### Join octopus network 

1. Register a Near account. You can refer to the [document](https://docs.near.org/docs/develop/basics/create-account) to complete the registration of the Near test network account;
2. Require OCT tokens. You need to join the [Discord](https://discord.gg/6GTJBkZA9Q) of the Octopus, and apply for the OCT tokens for testing in the **#testnet** channel;
3. Register Appchain. Log in to Octopus [testnet](https://testnet.oct.network/) with the Near account registered in step 1, click the register button, and input the *Appchain Name, Runtime URL, Runtime Hash, Bond Token*, and then complete the registration. After the transaction is successfully executed, save the generated **appchain_id**;
4. Integrate **pallet-octopus-appchain**. Integrate Appchain according to the above **Integration** section;
5. Generate the chainspec file of Appchain. Refer to this [document](https://substrate.dev/docs/en/tutorials/start-a-private-network/customspec), fill in appchain_id (generated in step 3) and relay contract name (dev-1616825757804-8762183), generate chainspec and check the data settings in it;
6. Upload the chainspec file of Appchain. After uploading, contact the Octopus team to start the network on the [Discord](https://discord.gg/6GTJBkZA9Q) **#testnet** channel of the Octopus.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
