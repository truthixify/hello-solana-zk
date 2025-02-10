const fs = require("fs");
const ffjavascript = require("ffjavascript");
const { unstringifyBigInts } = ffjavascript.utils;

async function main() {
    const inputPath = process.argv[2];
    if (!inputPath) {
        throw new Error("Input path not specified");
    }
    
    const outputPath = process.argv[3] ? `${process.argv[3]}/verifying_key.rs` : "verifying_key.rs";
    const fileData = fs.readFileSync(inputPath, "utf8");
    const vk = JSON.parse(fileData);

    const formatBigInt = (value) => `"${unstringifyBigInts(value).toString()}"`;
    const formatG1 = (point) => `(Fq::from_str(${formatBigInt(point[0])}).unwrap(), Fq::from_str(${formatBigInt(point[1])}).unwrap())`;
    const formatG2 = (point) => `(
        Fq2::new(Fq::from_str(${formatBigInt(point[0][0])}).unwrap(), Fq::from_str(${formatBigInt(point[0][1])}).unwrap()),
        Fq2::new(Fq::from_str(${formatBigInt(point[1][0])}).unwrap(), Fq::from_str(${formatBigInt(point[1][1])}).unwrap())
    )`;

    const rustOutput = `use ark_groth16::VerifyingKey;
use ark_bn254::{G1Affine, G2Affine, Fq, Fq2, Bn254};
use ark_std::str::FromStr;

pub fn get_vkey_from_json() -> VerifyingKey<Bn254> {
    let alpha_g1 = G1Affine::new${formatG1(vk.vk_alpha_1)};
    let beta_g2 = G2Affine::new${formatG2(vk.vk_beta_2)};
    let gamma_g2 = G2Affine::new${formatG2(vk.vk_gamma_2)};
    let delta_g2 = G2Affine::new${formatG2(vk.vk_delta_2)};
    let gamma_abc_g1 = vec![
${vk.IC.map(ic => `        G1Affine::new${formatG1(ic)},`).join("\n")}
    ];
    VerifyingKey {
        alpha_g1,
        beta_g2,
        gamma_g2,
        delta_g2,
        gamma_abc_g1
    }
};
`;

    fs.writeFileSync(outputPath, rustOutput);
    console.log("✅ Rust verifying key written to", outputPath);
}

main().catch(console.error);
