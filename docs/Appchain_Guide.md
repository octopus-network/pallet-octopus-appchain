## Appchain Guide

In this tutorial we will learn and practice how to connect Appchain to the Octopus network.

* Register Appchain
* Integrate Appchain

### Register Appchain

**What you will be doing**

Before we even get started, let's lay out what we are going to do over the course of this tutorial. We will:

* Create a Near account
* Request OCT tokens
* Register Appchain

#### Create a Near Account

We know that Octopus is an Appchains network running on the Near network. Therefore, you firstly need to have an account on the Near network before you can connect Appchain to the Octopus network.

The easiest way to create an account on NEAR is to use the NEAR wallet. Here we create a testnet account, navigate to https://wallet.testnet.near.org, and then click "Create Account", the next step, enter the desired account name.

More details [Create Near Account](https://docs.near.org/docs/develop/basics/create-account).

#### Request OCT tokens

This step is easy, just join the [Discord](https://discord.gg/6GTJBkZA9Q) of the Octopus network, in the *#testnet* channel, you can apply for the OCT tokens from the Octopus team.

#### Register Appchain

From Octopus network's architecture, we know that there is a relay contract. You can send a registration transaction with Appchain information by your Near account, which would interact with the relay contract and return the Appchain Id.

Log in to Octopus [testnet](https://testnet.oct.network/) with the Near testnet account, click the **Register** button, and enter *Appchain Name, Bond Token*, and then click “Register”, once the transaction is successfully executed, you can get the ID as your **appchain_id**.

### Integrate Appchain

**What you will be doing**

Before we even get started, let's lay out what we are going to do over the course of this tutorial. We will:

* Update Appchain runtime codes
* Generate and update Chain Spec file
* Provide the link and hash of Chain Spec file

#### Update Appchain runtime codes

The first step to integrate Appchain is to introduce the pallet `pallet-octopus-appchain` and update some codes of the substrate based blockchain.

1. Simply to add the following dependencies to the runtime's `Cargo.toml` file:

```TOML
[dependencies]
pallet-session = { default-features = false, version = '3.0.0' }
pallet-octopus-appchain = { default-features = false, git = 'https://github.com/octopus-network/pallet-octopus-appchain.git' }
```

and update the runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-session/std',
    'pallet-octopus-appchain/std',
]
```

2. And then to edit the runtime `lib.rs`, you should implement its trait like so:

```rust
parameter_types! {
	pub const AppchainId: pallet_octopus_appchain::ChainId = 3;
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
	const RELAY_CONTRACT_NAME: &'static [u8] = b"dev-1618284355026-5339538";
	type GracePeriod = GracePeriod;
	type UnsignedPriority = UnsignedPriority;
}
```

Change the value of constant **AppchainId** with your Appchain ID, and you can double check it from the Octopus [testnet](https://testnet.oct.network/).

```Rust
pub const AppchainId: pallet_octopus_appchain::ChainId = 3;
```

And you can get the value of **RELAY_CONTRACT_NAME** from the Octopus [testnet](https://testnet.oct.network/), e.g. Relay contract: dev-1618284355026-5339538

```Rust
const RELAY_CONTRACT_NAME: &'static [u8] = b"dev-1618284355026-5339538";
```

3. Include it in your `construct_runtime!` macro:

```rust
OctopusAppchain: pallet_octopus_appchain::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},
```

You can see the last commit of [Barnacle](https://github.com/octopus-network/barnacle) for a whole changes.

#### Generate and update Chain Spec file

1. Firstly, you need to generate your chainspec file, e.g.:

   ```bash
   ./target/debug/node-template build-spec --disable-default-bootnode --chain local > chain-spec.json
   ```

    More details [Create a Custom Chain Spec](https://substrate.dev/docs/en/tutorials/start-a-private-network/customspec)

2. Then, you can download the the chainspec snippet from the Octopus [testnet](https://testnet.oct.network/).

3. For your chainspec file, update the below fields with the related content from the chainspec snippet.

   * `palletBalance`
   * `palletSession`
   * `palletOctopusAppchain`

   Then generate the raw chainspec, e.g.:

   ```bash
   ./target/debug/node-template build-spec --chain=chain-spec.json --raw --disable-default-bootnode > chain-spec-raw.json
   ```

4. Generate the hash of the chainspec file, e.g.:

   ```bash
   # unix
   sha256sum chain-spec-raw.json
   ```

#### Provide the link and hash of Chain Spec file

To start the network, we need the chainspec file, so you can uploads it to your network space, and log in to Octopus [testnet](https://testnet.oct.network/) to fill in the file URL and hash.

Finally, to start the network, you can contact the Octopus team on the [Discord](https://discord.gg/6GTJBkZA9Q) #testnet channel of the Octopus network. 