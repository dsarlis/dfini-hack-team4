# dfini-hack-team4

## Prerequisites
Ensure that the following are provided
1. Rustup is installed
```
rustup update
```
2. Make sure the following target is added to rustup
```
rustup target add wasm32-unknown-unknown
```

Install didc from https://github.com/dfinity/candid/releases.

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

### Testing workflows

Start a local replica and install the canister:

```bash
dfx start --clean
dfx canister create dfini_hack_team4
dfx build
dfx canister install dfini_hack_team4
```

Register a user and submit a task:
```bash
dfx canister --no-wallet call dfini_hack_team4 register
```

Now prepare the blob argument for `submit_task`:
```bash
didc encode '(record {input = "Hello, world"; language = variant {german}})' --format blob
```

And finally call submit_task where `bytes` is the output of the above command:
```bash
dfx canister --no-wallet call dfini_hack_team4 submit_task '(variant {translate_text}, blob "bytes", 120000000000, 10)'
```

You can answer a task by:
```bash
dfx canister --no-wallet call dfini_hack_team4 answer_task '(0, blob "Hallo, welt")'
```

You can vote on an answer by:
```bash
dfx canister --no-wallet call dfini_hack_team4 vote '(0, 0, variant {yes})'
```