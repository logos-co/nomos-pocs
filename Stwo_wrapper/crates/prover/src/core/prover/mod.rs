use std::array;

use thiserror::Error;
use tracing::{span, Level};

use super::air::{Component, ComponentProver, ComponentProvers, Components};
use super::backend::BackendForChannel;
use super::channel::MerkleChannel;
use super::fields::secure_column::SECURE_EXTENSION_DEGREE;
use super::fri::FriVerificationError;
use super::pcs::{CommitmentSchemeProof, TreeVec};
use super::vcs::ops::MerkleHasher;
use crate::core::backend::CpuBackend;
use crate::core::channel::Channel;
use crate::core::circle::CirclePoint;
use crate::core::fields::qm31::SecureField;
use crate::core::pcs::{CommitmentSchemeProver, CommitmentSchemeVerifier};
use crate::core::poly::circle::CircleEvaluation;
use crate::core::poly::BitReversedOrder;
use crate::core::vcs::verifier::MerkleVerificationError;

#[derive(Debug)]
pub struct StarkProof<H: MerkleHasher> {
    pub commitments: TreeVec<H::Hash>,
    pub commitment_scheme_proof: CommitmentSchemeProof<H>,
}

#[derive(Debug)]
pub struct AdditionalProofData {
    pub composition_polynomial_oods_value: SecureField,
    pub composition_polynomial_random_coeff: SecureField,
    pub oods_point: CirclePoint<SecureField>,
    pub oods_quotients: Vec<CircleEvaluation<CpuBackend, SecureField, BitReversedOrder>>,
}

pub fn prove<B: BackendForChannel<MC>, MC: MerkleChannel>(
    components: &[&dyn ComponentProver<B>],
    channel: &mut MC::C,
    commitment_scheme: &mut CommitmentSchemeProver<'_, B, MC>,
) -> Result<StarkProof<MC::H>, ProvingError> {
    let component_provers = ComponentProvers(components.to_vec());
    let trace = commitment_scheme.trace();

    // Evaluate and commit on composition polynomial. (Hash(commitment[0])
    let random_coeff = channel.draw_felt();

    let span = span!(Level::INFO, "Composition").entered();
    let span1 = span!(Level::INFO, "Generation").entered();
    // Linear random combination of the trace polynomials
    let composition_polynomial_poly =
        component_provers.compute_composition_polynomial(random_coeff, &trace);
    span1.exit();

    // Before tree_builder is only column polynomial
    let mut tree_builder = commitment_scheme.tree_builder();
    // Extend it with composition polynomial
    tree_builder.extend_polys(composition_polynomial_poly.to_vec());
    // Reevaluate every polynomial (including composition) and absorb the root (Transcript <-- root)
    tree_builder.commit(channel);
    span.exit();

    // Draw OODS point.
    let oods_point = CirclePoint::<SecureField>::get_random_point(channel);

    // Get mask sample points relative to oods point.
    let mut sample_points = component_provers.components().mask_points(oods_point);
    // Add the composition polynomial mask points.
    sample_points.push(vec![vec![oods_point]; SECURE_EXTENSION_DEGREE]);

    // Prove the trace and composition OODS values, and retrieve them.
    let commitment_scheme_proof = commitment_scheme.prove_values(sample_points, channel);

    let sampled_oods_values = &commitment_scheme_proof.sampled_values;
    let composition_oods_eval = extract_composition_eval(sampled_oods_values).unwrap();

    // Evaluate composition polynomial at OODS point and check that it matches the trace OODS
    // values. This is a sanity check.
    if composition_oods_eval
        != component_provers
            .components()
            .eval_composition_polynomial_at_point(oods_point, sampled_oods_values, random_coeff)
    {
        return Err(ProvingError::ConstraintsNotSatisfied);
    }

    Ok(StarkProof {
        commitments: commitment_scheme.roots(),
        commitment_scheme_proof,
    })
}

pub fn verify<MC: MerkleChannel>(
    components: &[&dyn Component],
    channel: &mut MC::C,
    commitment_scheme: &mut CommitmentSchemeVerifier<MC>,
    proof: StarkProof<MC::H>,
) -> Result<(), VerificationError> {
    let components = Components(components.to_vec());
    let random_coeff = channel.draw_felt();

    // Read composition polynomial commitment.
    commitment_scheme.commit(
        *proof.commitments.last().unwrap(),
        &[components.composition_log_degree_bound(); SECURE_EXTENSION_DEGREE],
        channel,
    );

    // Draw OODS point z.
    let oods_point = CirclePoint::<SecureField>::get_random_point(channel);

    // Get mask sample points relative to oods point.
    let mut sample_points = components.mask_points(oods_point);
    // Add the composition polynomial mask points.
    sample_points.push(vec![vec![oods_point]; SECURE_EXTENSION_DEGREE]);

    let sampled_oods_values = &proof.commitment_scheme_proof.sampled_values;
    // Compute h0(z) + i * h1(z) + u * h2(z) + u*i* h3(z)
    let composition_oods_eval = extract_composition_eval(sampled_oods_values).map_err(|_| {
        VerificationError::InvalidStructure("Unexpected sampled_values structure".to_string())
    })?;
    println!("composition_oods_eval = {:?}",composition_oods_eval);
    if composition_oods_eval
        // Compute
        != components.eval_composition_polynomial_at_point(
            oods_point,
            sampled_oods_values,
            random_coeff,
        )
    {
        return Err(VerificationError::OodsNotMatching);
    }

    commitment_scheme.verify_values(sample_points, proof.commitment_scheme_proof, channel)
}

/// Extracts the composition trace evaluation from the mask.
fn extract_composition_eval(
    mask: &TreeVec<Vec<Vec<SecureField>>>,
) -> Result<SecureField, InvalidOodsSampleStructure> {
    // Last part of the proof
    let mut composition_cols = mask.last().into_iter().flatten();

    let coordinate_evals = array::try_from_fn(|_| {
        //Each Secure element of the sampled values
        let col = &**composition_cols.next().ok_or(InvalidOodsSampleStructure)?;
        let [eval] = col.try_into().map_err(|_| InvalidOodsSampleStructure)?;
        Ok(eval)
    })?;

    // Too many columns.
    if composition_cols.next().is_some() {
        return Err(InvalidOodsSampleStructure);
    }
    // Computes [0] + [1] * i + [2] * u + [3] * i * u

    Ok(SecureField::from_partial_evals(coordinate_evals))
}

/// Error when the sampled values have an invalid structure.
#[derive(Clone, Copy, Debug)]
pub struct InvalidOodsSampleStructure;

#[derive(Clone, Copy, Debug, Error)]
pub enum ProvingError {
    #[error("Constraints not satisfied.")]
    ConstraintsNotSatisfied,
}

#[derive(Clone, Debug, Error)]
pub enum VerificationError {
    #[error("Proof has invalid structure: {0}.")]
    InvalidStructure(String),
    #[error("{0} lookup values do not match.")]
    InvalidLookup(String),
    #[error(transparent)]
    Merkle(#[from] MerkleVerificationError),
    #[error(
        "The composition polynomial OODS value does not match the trace OODS values
    (DEEP-ALI failure)."
    )]
    OodsNotMatching,
    #[error(transparent)]
    Fri(#[from] FriVerificationError),
    #[error("Proof of work verification failed.")]
    ProofOfWork,
}
