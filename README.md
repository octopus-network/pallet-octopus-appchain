# Octopus Appchain Pallet
[![crates.io](https://img.shields.io/crates/v/pallet-octopus-appchain.svg)](https://crates.io/crates/pallet-octopus-appchain)
[![Released API docs](https://docs.rs/pallet-octopus-appchain/badge.svg)](https://docs.rs/pallet-octopus-appchain)

This is a support component of [Octopus Network](https://oct.network/).

## Purpose

With this pallet, a chain built from substrate can join octopus network as an appchain.

An appchain can rent security from motherchain on demand.

## Overview of Octopus

Octopus Network is a host for Web3.0 applications in the form of independent blockchains, aka appchains. Octopus Network provides leased security, interoperability, infrastructure and many other useful and affordable services to appchains.

![Octopus Network Image](https://github.com/octopus-network/pallet-octopus-appchain/blob/master/docs/networkdiagram.png)

### Relay

Octopus Relay is a set of [smart contracts](https://github.com/octopus-network/octopus-relay-contract) running on NEAR blockchain, provides leased security to appchains, and make appchain interoperable with NEAR and other appchains.

### Appchain

Appchain is a Substrate-based blockchain that is building for a special decentralized web application. A Substrate-based blockchain could integrate the pallet [pallet-octopus-appchain](https://github.com/octopus-network/pallet-octopus-appchain) as Octopus Appchain, and then [join the Octopus Network](https://github.com/octopus-network/pallet-octopus-appchain/blob/master/docs/Appchain_Guide.md) to get flexible and affordable leased security, powerful out-of-box interoperability, and many useful infrastructures. 

### Validator

Octopus Validator provide security to a appchain by staking OCT token into Octopus Relay and [running a validator]() node for this appchain.

All validator nodes for one appchain will form a quorum to reach consensus on block production, they would be rewarded by appchain native token issuing, and malicious actors will be slashed with the staked OCT. By this mechanism, appchain’s security is economically ensured by OCT staking in Octopus Relay.

## Appchain Guide

In this [tutorial](https://github.com/octopus-network/pallet-octopus-appchain/blob/master/docs/Appchain_Guide.md) we will learn and practice how to connect Appchain to the Octopus network.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
