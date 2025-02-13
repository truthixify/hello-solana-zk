# Solana Groth16 Verifier

A Groth16 zero-knowledge proof verifier implemented on Solana blockchain.

## Overview
This project provides a verifier that verifies zero-knowledge proofs generated using the Groth16 proving system. Groth16 is a widely used zk-SNARK (zero-knowledge Succinct Non-Interactive Argument of Knowledge) proof system that allows efficient and private verification of computations.

## Features
- Implements Groth16 proof verification on Solana.
- Optimized for Solana's BPF runtime.
- Securely verifies zk-SNARK proofs on-chain.

## Prerequisites
Ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)

## Installation
Clone the repository and navigate to the project directory:

```sh
git clone https://github.com/truthixify/hello-solana-zk.git
cd hello-solana-zk
```

## Build and test
### 1. Generate the verifier
```sh
npm install
node ./utils/generate_verifier.js <wasm_file> <zkey_file> <input_json> <verification_key_json>
```

### 2. Move the verifier to the solana program
```sh
mv ./verifier.rs ./program/src 
```

## Usage
You can interact with the verifier using a Solana client.

Example usage:
```rust
let proof = /* Load Groth16 proof */;
let public_inputs = /* Prepare public inputs */;
let mut data = vec![];
data.extend_from_slice(&proof);
data.extend_from_slice(&public_inputs);

let result = verifier::verify_proof(data);
assert!(result.is_ok());
```

## Testing
Run tests to ensure correctness:
```sh
cargo test-sbf
```

## License
This project is licensed under the MIT License.

## Acknowledgments
- [Solana Labs](https://solana.com/)
- [zk-SNARKs and Groth16](https://eprint.iacr.org/2016/260)
- [Light Protocol Groth16 Verifier](https://github.com/Lightprotocol/groth16-solana)