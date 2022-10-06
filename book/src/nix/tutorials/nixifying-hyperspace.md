# Nixifying Hyperspace

In this toturial, we will **nixify** a serivce written in Rust
that is currently not being built with Nix, and show how we can
[compose this service with Arion](../composing-services-with-arion.md).

## Checking out the repo

```bash
git clone git@github.com:ComposableFi/hyperspace
cd hyperspace
git checkout -b cor/nixify
```

## Adding a Nix Flake

Every nixified project should contain a `flake.nix` at the repository root, 
which describes which packages are included in this repository, as well as what a 
developer's shell should look like. 
[Read more about Flakes here](https://nixos.wiki/wiki/Flakes).

```bash
touch flake.nix
git add flake.nix
hx flake.nix # or vim, emacs, etc
```

_Note: `git add` is very important, as `nix` ignores all files that have not been
added to git_

We initialise our flake with a basic setup:

```nix
{
  description = "Hyperspace Relayer: Rust implementation of the IBC relayer algorithm.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils"; 
  };

  outputs = { self, nixpkgs, flake-utils }: {

  };
}
```

As you can see, we define two things here: a set of `inputs`, and a function that transforms
our `inputs` into our required outputs, named `outputs`.

- [nixpkgs](https://nixos.wiki/wiki/Nixpkgs) is the largest repository of Nix packages,
containing many essential (system) dependencies.
- [flake-utils](https://github.com/numtide/flake-utils) is a set of pure Nix flake utility functions,
making it a bit more ergonomic to write flake-based nix configurations.

## Creating cross-platform and cross-architecture packages

`flake-utils` makes it easy to generate nix package configurations for each system. 
This allows us to create packages for both **x86** and **ARM**, on both **Linux** and **macOS**.

```nix
{
  description = "Hyperspace Relayer: Rust implementation of the IBC relayer algorithm.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let pkgs = import nixpkgs { inherit system; }; in
    {
      packages = {
        hello = pkgs.hello;
      };
    });
}
```

Here we use the `flake-utils.lib.eachDefaultSystem` function in order to create a config for each `system`.
Within this function, we use the `system` to get the appropiate system-specific `pkgs` from `nixpkgs`.
Then, as an example, we define a package named `hello` which re-exports `pkgs.hello`.

Now, when typing `nix flake show`, we will see that we defined the hello package for all the desired systems:

```
git+file:///home/cor/dev/hyperspace?ref=refs%2fheads%2fcor%2fnixify&rev=b08ba2ef0abef204ff0cbf2d03d4ece3769aa4d3
└───packages
    ├───aarch64-darwin
    │   └───hello: package 'hello-2.12.1'
    ├───aarch64-linux
    │   └───hello: package 'hello-2.12.1'
    ├───i686-linux
    │   └───hello: package 'hello-2.12.1'
    ├───x86_64-darwin
    │   └───hello: package 'hello-2.12.1'
    └───x86_64-linux
        └───hello: package 'hello-2.12.1'
```

We can now run `nix run ".#hello"` and see "Hello World!" in our temrinal.
