const BN = require("bn.js");

function to32ByteLE(numStr) {
    return Buffer.from(new BN(numStr).toArray("le", 32));
}

function serializeProofAndPublicInputs(proof, publicInputs) {
    let buffer = Buffer.concat([
        to32ByteLE(proof.pi_a[0]),
        to32ByteLE(proof.pi_a[1]),

        to32ByteLE(proof.pi_b[0][1]), // b_x1
        to32ByteLE(proof.pi_b[0][0]), // b_x0
        to32ByteLE(proof.pi_b[1][1]), // b_y1
        to32ByteLE(proof.pi_b[1][0]), // b_y0

        to32ByteLE(proof.pi_c[0]),
        to32ByteLE(proof.pi_c[1]),

    ]);

    return buffer;
}

function formatRustArray(buffer) {
    let rustArray = `pub const PROOF: [u8; ${buffer.length}] = [\n    `;
    rustArray += buffer.toJSON().data.join(", ");
    rustArray += "\n];";
    return rustArray;
}

// Load proof.json and public.json
const proof = require("./proof.json");
const publicInputs = require("./public.json");

// Serialize everything into a buffer
const serializedData = serializeProofAndPublicInputs(proof, publicInputs);

// Generate Rust-friendly output
console.log(formatRustArray(serializedData));