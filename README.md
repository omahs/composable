---
description: >-
  Built on Kusama, Polkadot’s canary network, the composable parachain called
  Picasso offers enhanced interoperability, customization, and security over
  traditional blockchain structures.
cover: .gitbook/assets/Screen Shot 2022-01-28 at 1.34.46 PM.png
coverY: 96.26075504828798
---

# Composable Parachain

## Getting Started!

Welcome to Picasso Parachain wiki! Here you'll find everything you need to know about operating a composable parachain node.



## Introduction

Running a blockchain node can be a bit of hardwork and as a result, we recommend you have a technical background to help you navigate this better, However to make this process easier for you, we have automated a lot of the tedious process using Terraform and Ansible, you can find access to any of the resources you need here and choose the one that best suits your need

Ansible - [https://github.com/ComposableFi](https://github.com/ComposableFi/SRE)/ansible

Terraform - [https://github.com/ComposableFi](https://github.com/ComposableFi/SRE)/terraform

Running a composable full node allows you access to connect to the full network to sync with a bootnode, get information from the rpc endpoints and even author blocks on the parachain yourself.

To setup a full composable node, you will need to complete the following steps:

* Provision the infrastructure the nodes will run on
* Build or Download composable latest code
* Run your composable binary to get your node started
* Monitor your node(s)

We have 3 major environments for our network and they are illustrated below

| Chain    | Network Hosted on | Name           |
| -------- | ----------------- | -------------- |
| Rococo   | Centrifuge        | Dali-chachacha |
| Kusama   | Polkadot          | Picasso        |
| Polkadot | Polkadot          | Composable     |



## Provision Node Infrastructure

You can provision your virtual machine using any cloud service provider of your choice or on your bare metal vm's, whatever rocks your boat, however for better performance we suggest the following requirements:

### Hardware Requirements:&#x20;

| Node Type  | CPU     | Memory | Disk Size |   |
| ---------- | ------- | ------ | --------- | - |
| Validators | 4 cores | 16GB   | 200 GB    |   |
| Collators  | 8 cores | 64GB   | 200 GB    |   |
| Bootnode   | 4 cores | 32GB   | 200 GB    |   |
| RPC        | 2 cores | 32GB   | 200 GB    |   |

_Note: The table above is an estimation and does not mean you necessarily need to have all this specifications before you can get a node running_

* CPU - Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz
* Storage - A NVMe solid state drive. Should be reasonably sized to deal with blockchain growth. Starting around 80GB - 160GB will be okay for the first six months of Polkadot, but will need to be re-evaluated every six months.
* Memory - 64GB.
* Operating System: All Linux based OS are supported, for the purpose of this guide, we will be using Ubuntu18.04

Note: The above requirements represents our current best estimate

### Networking Requirements:

All nodes will require a static IPv4 address or a valid DNS name that points to your node, Firewall settings should follow this recommendations:\
Inbound Traffic: TCP/443 for grpc

OutBound Traffic: TCP/30333, TCP/30334 for bootnodes peering



## Setting up Node

If you are building crowd loan product on Composable, you will need to run your own node, this guide will show you how to setup your node in a highly scalable way and also all you need to know to maintain the nodes and make them highly available including setting up a full blown monitoring infrastructure.

Before we dive into creating nodes, let us first of all understand the different kind of nodes we will be creating, and get to understand its purpose

### Collator Node

Collators nodes are responsible for running a full node for both the composable parachain and its relaychain, they produce the state transition proof for relay chain validators.

Spinning up a collator node on the composable network is quite simple, you can follow the technical requirements above for what's needed, please note that collator node requirs higher cpu resources for higher transaction throughput

to run collator nodes:

```
// Some code
composable --collator 
```

### Boot Node

Bootnodes are regular nodes used to discover other peer of nodes,&#x20;

#### Configuring Bootnodes

Each bootnode has a peer ID, the peer id is automatically generated when the composable binary is run, however becuase the peer ID is generated everytime the composable binary is run, this means that the peer id will change often, to have a static peer id that doesnt change, use this command to generate a node key that your bootnode can use as its peer id

```
// Some code
parachain-utils genrate keys 
```



### RPC Node

All decentralized applications need a way to communoicate with their blockchains, this is what allows access to informations that the blockchain transacts on, this connection is faciliated by the RPC nodes, RPC is short form for Remote Procedure Call, and they are a form of interprocess communication,&#x20;

RPC nodes also run as an archive nodes, this mean they keeps all the state of the past blocks, archive node makes it convenient to query the past state of the chain at any point in time.&#x20;

In composable network, the rpc node client exposes https and wsss endpoint for rpc connections using port 9933 and 9944 respectively, to run a node as an rpc client, run the following commands

```
// Some code
composable --rpc-port
```

### Setup Data Disks

Composable&#x20;

### Generating Node Keys



#### Setup dependencies

#### Install Rust and its dependencies

Once you have setup your VM and its fully up and running, the next step is to get all the dependencies that is needed to run or build a composable node from scratch ready, composable is fully written in RUST, this means we need to have rust and its dependencies setup on our vm, if you have never installed RUst, it's okay, follow the instruction below on how to setup rust on your vm

```
#!/bin/bash
sudo apt install make clang pkg-config libssl-dev build-essential
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env
rustc --version



```

However, uf you already have rust installed, you can proceed to this next step to byild composable binary from source

#### Time Synchronization

NTP is a networking protocol designed to synchronize the clocks of computers over a network, this allows you to synchronize the clocks of all the syste,s within the network so that all networks and nodes stay in sync





### Building and Installing a Composable Binary

There are different ways to get started running a composable node:

1. Installing composable with a Package Manager
2. Using already built composable binary
3. Building composable Binary from source

### 1. Installing composable with a package manager

{% tabs %}
{% tab title="Debian" %}

{% endtab %}

{% tab title="RPM" %}

{% endtab %}
{% endtabs %}



### Starting Node with Composable Binary



### Generate Session Keys

### Using Docker



### Monitoring and Logging for Picasso Nodes



### Securing Picasso Nodes







