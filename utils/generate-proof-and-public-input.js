const snarkjs = require('snarkjs');
const fs = require('fs');

async function generateProof() {
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
        { a: 3, b: 4 }, // Inputs
        './utils/Multiplier.wasm', // WASM file
        './utils/circuit_final.zkey' // Proving key
    );

    fs.writeFileSync('proof.json', JSON.stringify(proof));
    fs.writeFileSync('public.json', JSON.stringify(publicSignals));
    console.log('Generated proof and public files.');
}

generateProof().catch(console.error);
