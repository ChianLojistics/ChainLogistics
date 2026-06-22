use soroban_sdk::crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr};
use soroban_sdk::{contract, contractimpl, Bytes, BytesN, Env, Vec};

#[contract]
pub struct ComplianceVerifier;

#[contractimpl]
impl ComplianceVerifier {
    /// Verifies a Groth16 proof for the BN254 curve.
    pub fn verify(
        env: Env,
        proof_a: Bytes,
        proof_b: Bytes,
        proof_c: Bytes,
        public_inputs: Vec<Bytes>,
        vk_alpha_g1: Bytes,
        vk_beta_g2: Bytes,
        vk_gamma_g2: Bytes,
        vk_delta_g2: Bytes,
        vk_ic: Vec<Bytes>,
    ) -> bool {
        // Convert Bytes to BN254 types
        let proof_a = Bn254G1Affine::from_bytes(proof_a.try_into().unwrap());
        let proof_b = Bn254G2Affine::from_bytes(proof_b.try_into().unwrap());
        let proof_c = Bn254G1Affine::from_bytes(proof_c.try_into().unwrap());

        let vk_alpha_g1 = Bn254G1Affine::from_bytes(vk_alpha_g1.try_into().unwrap());
        let vk_beta_g2 = Bn254G2Affine::from_bytes(vk_beta_g2.try_into().unwrap());
        let vk_gamma_g2 = Bn254G2Affine::from_bytes(vk_gamma_g2.try_into().unwrap());
        let vk_delta_g2 = Bn254G2Affine::from_bytes(vk_delta_g2.try_into().unwrap());

        // Enforce public inputs match VK IC length
        if public_inputs.len() + 1 != vk_ic.len() {
            return false;
        }

        // 1. Compute VK_x = IC[0] + sum(public_inputs[i] * IC[i+1])
        let mut vk_x = Bn254G1Affine::from_bytes(vk_ic.get(0).unwrap().try_into().unwrap());

        for i in 0..public_inputs.len() {
            let input_scalar = Fr::from_bytes(public_inputs.get(i).unwrap().try_into().unwrap());
            let ic_point = Bn254G1Affine::from_bytes(vk_ic.get(i + 1).unwrap().try_into().unwrap());

            let scaled = env.crypto().bn254().g1_mul(&ic_point, &input_scalar);
            vk_x = env.crypto().bn254().g1_add(&vk_x, &scaled);
        }

        // 2. Prepare pairing check: e(A, B) * e(-vk_alpha, vk_beta) * e(-vk_x, vk_gamma) * e(-proof_c, vk_delta) == 1

        let neg_vk_alpha = -vk_alpha_g1;
        let neg_vk_x = -vk_x;
        let neg_proof_c = -proof_c;

        let g1_points = Vec::from_array(&env, [proof_a, neg_vk_alpha, neg_vk_x, neg_proof_c]);

        let g2_points = Vec::from_array(&env, [proof_b, vk_beta_g2, vk_gamma_g2, vk_delta_g2]);

        env.crypto().bn254().pairing_check(g1_points, g2_points)
    }
}
