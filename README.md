# Open rollup Pallet

Open rollup pallet provides a general zk-rollup framework for all substrate-based blockchains.

## Features

- There is no central operator here, developers can freely register their zkapp on the chain, implement their own offline program, and submit their ZK Proofs and state changes for their batch transactions.

- Support Currency, Fungible, Nonfungible tokens in Substrate.

- Miden verifier has been integrated to support the verification of the [miden](https://github.com/0xPolygonMiden/miden-vm) program.

## Protocol

- **Zkapp registration.** Developers use a program-hash to register a zkapp. The program-hash is derived from the bytecode of the zkapp. If the zkapp is open source, users can verify that the program-hash is indeed corresponding to the zkapp. 
- **User deposit.** Users who want to participate in a zkapp deposit to this zkapp, and the batch submitted by this zkapp needs to include it.
- **User withdraw.** Users can submit an withdraw transaction, and this zkapp include it in the next submission.
- **User Move Asset.** Users move their assets from a zkapp to another zkapp, and this zkapp include it in the next submission. 
- **User full exit.** If the zkapp status is inactive, the user can exit the zkapp fully and withdraw their assets.
- **Zkapp batch submit.** Submit a batch for a zkapp, can only be called by submitter of the zkapp.

## Tests

Use Rust's native cargo command

```bash
cargo test --all-features
```

or in docker:

```bash
docker build -t open-rollup .
docker run --rm open-rollup
```

## Documentation

The in-code documentation can be opened with:

```bash
cargo doc --no-deps --open --package pallet-open-rollup
```

## Run a Substrate node with Open Rollup pallet

```bash
git clone https://github.com/open-rollup/open-rollup-node
cargo run --release -- --dev
```

Then can use [substrate-front-end-template](https://github.com/substrate-developer-hub/substrate-front-end-template) to try out the pallet.

