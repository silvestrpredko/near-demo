# near-demo

## Task
The contract should include the following methods:
1. Initialization method:
Input are the address of the contract owner and the addresses of two tokens (hereinafter token A and token B).

\- The method requests and stores the metadata of tokens (name, decimals) ✅

\- Creates wallets for tokens А & В. ✅

2. The method for getting information about the contract (ticker, decimals, ratio of tokens A and B) ✅
3. Deposit method

\- The user can transfer a certain number of tokens A to the contract account and in return
   must receive a certain number of tokens B (similarly in the other direction).  ✅

\- The contract supports a certain ratio of tokens A and B. X * Y = K (K is some constant value, X and Y
   are the number of tokens A and B respectively. ✅

\- The owner of the contract can transfer a certain amount of tokens A or B to the contract account, thereby changing the ratio K. ✅

Implementation requirements in order of their priority.

\- Implement contact. The contract must work with two tokens with ✅
 **an arbitrary number of decimals**. ❎

> Currently, contract doesn't work with decimals, I didn't cover this concept yet

\- Smart contact should be tested. ✅

\- Instructions in the readme: contract building, deployment, creation of a token, contract initialization, contract testing description. ✅

## How to build

1. First of all please install Rust toolchain.
2. Add `wasm32-unknown-unknown` target to compiler
```sh 
rustup target add wasm32-unknown-unknown
```
3. Build contracts.
```sh
cargo build-contracts 
```

## How to run tests

For this demo was implemented integration tests that are used a `Near` [workspaces](https://github.com/near/workspaces-rs)

Please take a look on `integration-tests`

```sh
cargo run-tests
```
