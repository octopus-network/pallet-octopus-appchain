## Appchain Guide

In this tutorial, we will learn and practice how to connect your Appchain to the Octopus network.

* Register your Appchain
* Integrate your Appchain

### Register your Appchain

**What you will be doing**

* Create a NEAR account
* Request OCT tokens
* Register your Appchain

#### Create a Near Account

Octopus network is a network of Appchains running on the NEAR blockchain network. Therefore, you need to have an account on the NEAR network before you can connect your Appchain to the Octopus network.

The easiest way to create an account on NEAR is to use the NEAR wallet. Go to https://wallet.testnet.near.org, and follow the instructions to create your account.

More details [Create NEAR Account](https://docs.near.org/docs/develop/basics/create-account).

#### Request OCT tokens

Join our [Discord server] https://discord.gg/6GTJBkZA9Q  
In the *#testnet* channel, enter "apply for OCT tokens".
Someone from our team will be in touch with you shortly.

#### Register your Appchain

From the Octopus network's architecture, we know there is a relay contract. You can send a registration transaction with your Appchain information using your NEAR account, which would interact with the relay contract and return your ```appchain_id```

Log in to Octopus [testnet](https://testnet.oct.network/) with your NEAR testnet account. Click the **Register** button, enter *Appchain Name, Bond Token*, then click “Register”. Once the transaction has been successfully executed, you will get the ID as your **appchain_id**.

### Integrate your Appchain

**What you will be doing**

* Update your Appchain runtime code
* Generate and update the **Chain Spec** file
* Provide the link and hash of the **Chain Spec** file

#### Update your Appchain runtime code

The first step to integrate your Appchain is to introduce the pallet `pallet-octopus-appchain` and update the code of the substrate-based blockchain.

1. Add the following dependencies to the runtime's `Cargo.toml` file:

```TOML
[dependencies]
pallet-session = { default-features = false, version = '3.0.0' }
pallet-octopus-appchain = { default-features = false, git = 'https://github.com/octopus-network/pallet-octopus-appchain.git' }
```

Update the runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-session/std',
    'pallet-octopus-appchain/std',
]
```

2.Edit the runtime `lib.rs`, you should implement its trait like this:

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

Change the value of constant **AppchainId** with your Appchain ID, and you can find it from the Octopus [testnet](https://testnet.oct.network/).

```Rust
pub const AppchainId: pallet_octopus_appchain::ChainId = 3;
```

You can get the value of **RELAY_CONTRACT_NAME** from the Octopus [testnet](https://testnet.oct.network/), e.g. Relay contract: dev-1618284355026-5339538

```Rust
const RELAY_CONTRACT_NAME: &'static [u8] = b"dev-1618284355026-5339538";
```

3. Include it in your `construct_runtime!` macro:

```rust
OctopusAppchain: pallet_octopus_appchain::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},
```

You can see the last commit of [Barnacle](https://github.com/octopus-network/barnacle) for the entire changes.

#### Generate and update the Chain Spec file

1. First, you need to generate your chainspec file, e.g.:

   ```bash
   ./target/debug/node-template build-spec --disable-default-bootnode --chain local > chain-spec.json
   ```

    More details [Create a Custom Chain Spec](https://substrate.dev/docs/en/tutorials/start-a-private-network/customspec)

2. You can download the chainspec snippet from the Octopus [testnet](https://testnet.oct.network/).

3. For your chainspec file, update these fields with the related content from the chainspec snippet.

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

To start the network, we need the chainspec file. You can upload it to your network space, and logon to Octopus [testnet](https://testnet.oct.network/) to fill in the file URL and hash.

Finally, to start the network, you can contact the Octopus team on the [Discord](https://discord.gg/6GTJBkZA9Q) #testnet channel of the Octopus network. 

### 🎉🎉🎉 Yay Congratulations! Excited to see what we could build together using this exciting technology! If you have anymore questions, please feel free to chat with us on Discord! We will reply to every message from you 🤟
