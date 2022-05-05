# /!\ Install Nix + Flakes

1. https://nixos.org/download.html
2. https://nixos.wiki/wiki/Flakes

# Run locally

-  `./update.sh REVISION` where `REVISION` is the latest deployed commit hash.
- From root directory run:
``shell
nix develop .#devnet
```
or 
```shell
nix develop .#devnet --extra-experimental-features nix-command --extra-experimental-features flakes
```

- then then run the devnet using `run-dali-dev`.

- Reach alice at `https://polkadot.js.org/apps/?rpc=ws://localhost:9944#/explorer`

# GCE

/!\ Download your GCE service account key and save it as `ops.json`.

## Deploy

1. `nix develop .#deploy`
2. `nixops create -d devnet-gce`
3. `nixops deploy -d devnet-gce`

## Connect to CI deployed machines

1. Download nixops CI state: `gsutil cp gs://composable-state/deployment.nixops .`
2. Run `NIXOPS_STATE=deployment.nixops nixops ssh composable-devnet-dali-dev`
