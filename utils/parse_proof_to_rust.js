import { buildBn128, utils } from 'ffjavascript';
const { unstringifyBigInts } = utils;
import { createRequire } from 'module';
const require = createRequire(import.meta.url);
const fs = require('fs');

function to32ByteBuffer(bigInt) {
    const hexString = bigInt.toString(16).padStart(64, '0'); // Pad to 64 hex characters (32 bytes)
    const buffer = Buffer.from(hexString, 'hex');
    return buffer;
}

function g1Uncompressed(curve, p1Raw) {
    let p1 = curve.G1.fromObject(p1Raw);

    let buff = new Uint8Array(64); // 64 bytes for G1 uncompressed
    curve.G1.toRprUncompressed(buff, 0, p1);

    return Buffer.from(buff);
}

// Function to negate G1 element
function negateG1(curve, buffer) {
    let p1 = curve.G1.fromRprUncompressed(buffer, 0);
    let negatedP1 = curve.G1.neg(p1);
    let negatedBuffer = new Uint8Array(64);
    curve.G1.toRprUncompressed(negatedBuffer, 0, negatedP1);
    return Buffer.from(negatedBuffer);
}

// Function to reverse endianness of a buffer
function reverseEndianness(buffer) {
    return Buffer.from(buffer.reverse());
}

async function negateAndSerializeG1(curve, reversedP1Uncompressed) {
    if (
        !reversedP1Uncompressed ||
        !(
            reversedP1Uncompressed instanceof Uint8Array ||
            Buffer.isBuffer(reversedP1Uncompressed)
        )
    ) {
        console.error(
            'Invalid input to negateAndSerializeG1:',
            reversedP1Uncompressed
        );
        throw new Error('Invalid input to negateAndSerializeG1');
    }
    // Negate the G1 point
    let p1 = curve.G1.toAffine(
        curve.G1.fromRprUncompressed(reversedP1Uncompressed, 0)
    );
    let negatedP1 = curve.G1.neg(p1);

    // Serialize the negated point
    // The serialization method depends on your specific library
    let serializedNegatedP1 = new Uint8Array(64); // 32 bytes for x and 32 bytes for y
    curve.G1.toRprUncompressed(serializedNegatedP1, 0, negatedP1);
    // curve.G1.toRprUncompressed(serializedNegatedP1, 32, negatedP1.y);

    // Change endianness if necessary
    let proof_a = reverseEndianness(serializedNegatedP1);

    return proof_a;
}

function g2Uncompressed(curve, p2Raw) {
    let p2 = curve.G2.fromObject(p2Raw);

    let buff = new Uint8Array(128); // 128 bytes for G2 uncompressed
    curve.G2.toRprUncompressed(buff, 0, p2);

    return Buffer.from(buff);
}

export async function proofData(proof, publicSignals) {
    let curve = await buildBn128();
    let proofProc = unstringifyBigInts(proof);
    publicSignals = unstringifyBigInts(publicSignals);

    let pi_a = g1Uncompressed(curve, proofProc.pi_a);
    pi_a = reverseEndianness(pi_a);
    pi_a = await negateAndSerializeG1(curve, pi_a);

    const pi_b = g2Uncompressed(curve, proofProc.pi_b);

    const pi_c = g1Uncompressed(curve, proofProc.pi_c);

    // Assuming publicSignals has only one element
    const publicSignalsBuffer = to32ByteBuffer(BigInt(publicSignals));

    const serializedData = Buffer.concat([
        pi_a,
        pi_b,
        pi_c,
        publicSignalsBuffer,
    ]);

    return Array.from(serializedData);
}

export async function generateRustProof() {
    const inputPath1 = process.argv[2];
    const inputPath2 = process.argv[3];
    if (!inputPath1 || !inputPath2) {
        throw new Error('Input path not specified');
    }

    const outputPath = process.argv[4]
        ? `${process.argv[4]}/proof.rs`
        : 'proof.rs';
    const fileData1 = fs.readFileSync(inputPath1, 'utf8');
    const fileData2 = fs.readFileSync(inputPath2, 'utf8');
    const proof = JSON.parse(fileData1);
    const publicSignals = JSON.parse(fileData2);
    const data = await proofData(proof, publicSignals);
    const proofArr = data.slice(0, 256);
    const publicSignalsArr = data.slice(256);

    const rustOutput = `pub const PROOF: [u8; 256] = [${proofArr}];

pub const PUBLIC_SIGNALS: [[u8; 32]; 1] = [[${publicSignalsArr}]];
`;

    fs.writeFileSync(outputPath, rustOutput);
    console.log('✅ Rust proof written to', outputPath);
}

generateRustProof().catch(console.error);
