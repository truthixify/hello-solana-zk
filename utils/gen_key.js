const fs = require("fs");

// Load the verification key JSON file
const vkJson = JSON.parse(fs.readFileSync("./utils/verification_key.json", "utf8"));

// Extract the components of the verification key
const alpha_g1 = vkJson.vk_alpha_1;
const beta_g2 = vkJson.vk_beta_2;
const gamma_g2 = vkJson.vk_gamma_2;
const delta_g2 = vkJson.vk_delta_2;
const ic = vkJson.IC;

// Function to convert a serialized point to arkworks format
function convertG1(point) {
    return {
        x: point[0], // Fq element (x-coordinate)
        y: point[1], // Fq element (y-coordinate)
        infinity: false, // Assume the point is not at infinity
    };
}

function convertG2(point) {
    return {
        x: [point[0][0], point[0][1]], // Fq2 element (x-coordinate)
        y: [point[1][0], point[1][1]], // Fq2 element (y-coordinate)
        infinity: false, // Assume the point is not at infinity
    };
}

// Convert the verification key components
const alpha = convertG1(alpha_g1);
const beta = convertG2(beta_g2);
const gamma = convertG2(gamma_g2);
const delta = convertG2(delta_g2);
const ic_converted = ic.map((point) => convertG1(point));

// Construct the VerifyingKey object
const verifyingKey = {
    alpha_g1: alpha,
    beta_g2: beta,
    gamma_g2: gamma,
    delta_g2: delta,
    gamma_abc_g1: ic_converted,
};

// Save the VerifyingKey to a file or pass it to a Rust backend
fs.writeFileSync("verifying_key_ark.json", JSON.stringify(verifyingKey, null, 2));
console.log("VerifyingKey generated and saved to verifying_key_ark.json");