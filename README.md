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



### Operating System Requirements:

Composable binary is distributed as:

* An executable file targeting an amd64 architecture&#x20;
* A linux container image

> Later releases for composable binary will allow you the freedom to choose your own architecture,  this means that there will be most cross platform support for different OS architecture and you can modify to suit your needs as you please

### Networking Requirements:

All nodes will require a static IPv4 address or a valid DNS name that points to your node, Firewall settings should follow this recommendations:\
Inbound Traffic: TCP/443 for grpc

OutBound Traffic: TCP/30333, TCP/30334 for bootnodes peering

_Note: All nodes must have port 30333 and 30334 accessible to all_

| Port  | Protocol              |   |
| ----- | --------------------- | - |
| 30333 | Relaychain p2p        |   |
| 30334 | Parachain p2p         |   |
| 9944  | ws                    |   |
| 9933  | rpc                   |   |
| 9615  | relaychain prometheus |   |
| 9616  | parachain prometheus  |   |

__

## Setting up Node

### Node Prerequisities: Install Rust dependencies

This step will guide you through the steps you need to prepare and bake your composable binary in a way that you want for you to use in your node operations, the first thing you will need to do is setup your environment for rust, once that is done, you can then move ahead to building your binaries and distributing to any where you want

1. **Installing Rust Dependencies**

{% tabs %}
{% tab title="Debian" %}
```
sudo apt update
// Install the necessary dependencies for compiling composable binary
sudo apt install -y git clang curl libssl-dev llvm libudev-dev
// Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
{% endtab %}

{% tab title="Fedora" %}
```
sudo dnf update
sudo dnf install clang curl git openssl-devel
```
{% endtab %}
{% endtabs %}

2\. Setup Rust development environment

```
// Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
// Set path
source ~/.cargo/env
// verify rust installation
rustc --version
```

3\. Configure Rust toolchain to default to the latest stable version and add nightly target

```
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

4\. Confirm your setup

To see what Rust toolcchain you are presently using, run:

```
rustup show
```

this will show you an output like this

```
Default host: x86_64-unknown-linux-gnu
rustup home:  /home/user/.rustup

installed toolchains
--------------------

stable-x86_64-unknown-linux-gnu (default)
nightly-2020-10-06-x86_64-unknown-linux-gnu
nightly-x86_64-unknown-linux-gnu

installed targets for active toolchain
--------------------------------------

wasm32-unknown-unknown
x86_64-unknown-linux-gnu

active toolchain
----------------

stable-x86_64-unknown-linux-gnu (default)
rustc 1.50.0 (cb75ad5db 2021-02-10)
```

5\. Setup Time Synchronization&#x20;

NTP is a networking protocol designed to synchronize the clocks of computers over a network, this allows you to synchronize the clocks of all the systems within the network so that all networks and nodes stay in sync

You need to ensure you run time synchronization  on your nodes to avoid clock drift, all you have to do is to run an ntp protocol client as a daemon and configure it to point to an ntp server to query periodically. Most linux distributions may already have this installed and configured, however if not, you can run the following commands to do that:

Install NTP client

```
sudo apt-get install ntp
```

run this command to configure ntp

```
timedatactl
```

if ntp is installed and running, you should see an output like this: **System clock synchronized: yes,** otherwise it means there is no ntp client installed on your machine and you should go ahead and install it.

To query ntpd for status information in order to verifiy that everything is working fine, run::

```
sudo ntpq -p
```

### Building and Installing composable Binary

There are several ways to install composable binary:

1. Building from source
2. Downloading prebuilt binary
3. Installing with package manager

#### Building from source

To build composable binary, clone the source repository from our github repository [ComposableFi/composable](https://github.com/ComposableFi/composable.git)  and run the following command

clone the repository

```
git clone https://github.com/ComposableFi/composable.git
```

enter the composable directory and run the following command to find the latest version

```
RELEASE_VERSION=$(git tag --sort=committerdate | grep -E '^v[0-9]' | tail -1)
```

checkout to the latest release

```
git fetch && git checkout $RELEASE_VERSION
./scripts/init.sh 
```

build native code with the cargo release profile, the build time depends on how beefy your machine is, but usually about 15 - 45minutes tops

```
cargo build --release -p composable
```

### Downloading prebuilt binary

We provide pre-compiled binary executables for common operating systems and architectures

### Setting up Nodes

If you are building crowd loan product on Composable, you will need to run your own node, this guide will show you how to setup your node in a highly scalable way and also all you need to know to maintain the nodes and make them highly available including setting up a full blown monitoring infrastructure.

Before we dive into creating nodes, let us first of all understand the different kind of nodes we will be creating, and get to understand its purpose

### Running a Collator Node

Collators nodes are responsible for running a full node for both the composable parachain and its relaychain, they produce the state transition proof for relay chain validators.

Spinning up a collator node on the composable network is quite simple, you can follow the technical requirements above for what's needed, please note that collator node requires higher cpu resources for higher transaction throughput

to run collator nodes:

```
composable --collator  --chain=<chain>  --name <node-name> --base-path </path/to/data/dir>  
--listen-addr=/ip4/0.0.0.0/tcp/30334 --public-addr=/ip4/<public_ip>/tcp/30334  --execution=wasm  -- 
--execution=wasm  --listen-addr=/ip4/0.0.0.0/tcp/30333 --public-addr=/ip4/35.205.160.54/tcp/30333 -l gossip=debug,peerset=debug 
```

### Running a Boot Node

Bootnodes are regular nodes used to discover other peer of nodes,&#x20;

#### Configuring Bootnodes

Each bootnode has a peer ID, the peer id is automatically generated when the composable binary is run, however becuase the peer ID is generated everytime the composable binary is run, this means that the peer id will change often, to have a static peer id that doesnt change, use this command to generate a node key that your bootnode can use as its peer id

```
// Some code
parachain-utils genrate keys 
```



### Running an RPC Node

All decentralized applications need a way to communoicate with their blockchains, this is what allows access to informations that the blockchain transacts on, this connection is faciliated by the RPC nodes, RPC is short form for Remote Procedure Call, and they are a form of interprocess communication,&#x20;

RPC nodes also run as an archive nodes, this mean they keeps all the state of the past blocks, archive node makes it convenient to query the past state of the chain at any point in time.&#x20;

In composable network, the rpc node client exposes https and wsss endpoint for rpc connections using port 9933 and 9944 respectively, to run a node as an rpc client, run the following commands

```
// Some code
composable --rpc-port
```

Now that we have understood all different node types, let us proceed to getting our composable binary ready for our nodes operations

### Setup Data Disks

Composable stores all its state on disk, using a key-value store database called RocksDB, there are two types of states that are going to be stored on your disk:

* Relaychain Data
* Parachain Data

**Relaychain Data**

****

**Parachain Data**

****

**Confidential Data and How to treat them**

###

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

####





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







