# dfini-hack-team4

## Prerequisites
Ensure that the following this are provided
1. Rustup is installed
```
rustup update
```
2. Make sure the following target is added to rustup
```
rustup target add wasm32-unknown-unknown
```

## Local Development

After cloning the repository,

```bash
dfx start [--clean] [--background]
```

In a different terminal, create a canister id:

```bash
dfx canister create dfini_hack_team4
```

and build the canister

```bash
dfx build
```

