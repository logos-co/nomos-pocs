use itertools::Itertools;
use std::fs::File;
use std::io::{Error, Write};
use crate::constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval};
use crate::core::backend::simd::m31::{PackedBaseField, LOG_N_LANES};
use crate::core::backend::simd::SimdBackend;
use crate::core::backend::{Col, Column};
use crate::core::fields::m31::BaseField;
use crate::core::fields::FieldExpOps;
use crate::core::poly::circle::{CanonicCoset, CircleEvaluation};
use crate::core::poly::BitReversedOrder;
use crate::core::ColumnVec;
use crate::core::fields::qm31::SecureField;
use crate::core::prover::StarkProof;
//use crate::core::vcs::hash::Hash;
use crate::core::vcs::poseidon_bls_merkle::PoseidonBLSMerkleHasher;

pub type WideFibonacciComponent<const N: usize> = FrameworkComponent<WideFibonacciEval<N>>;

pub struct FibInput {
    a: PackedBaseField,
    b: PackedBaseField,
}

/// A component that enforces the Fibonacci sequence.
/// Each row contains a seperate Fibonacci sequence of length `N`.
#[derive(Clone)]
pub struct WideFibonacciEval<const N: usize> {
    pub log_n_rows: u32,
}
impl<const N: usize> FrameworkEval for WideFibonacciEval<N> {
    fn log_size(&self) -> u32 {
        self.log_n_rows
    }
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_n_rows + 1
    }
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let mut a = eval.next_trace_mask();
        let mut b = eval.next_trace_mask();
        for _ in 2..N {
            let c = eval.next_trace_mask();
            eval.add_constraint(c - (a.square() + b.square()));
            a = b;
            b = c;
        }
        eval
    }
}

pub fn generate_trace<const N: usize>(
    log_size: u32,
    inputs: &[FibInput],
) -> ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
    assert!(log_size >= LOG_N_LANES);
    assert_eq!(inputs.len(), 1 << (log_size - LOG_N_LANES));
    let mut trace = (0..N)
        .map(|_| Col::<SimdBackend, BaseField>::zeros(1 << log_size))
        .collect_vec();
    for (vec_index, input) in inputs.iter().enumerate() {
        let mut a = input.a;
        let mut b = input.b;
        trace[0].data[vec_index] = a;
        trace[1].data[vec_index] = b;
        trace.iter_mut().skip(2).for_each(|col| {
            (a, b) = (b, a.square() + b.square());
            col.data[vec_index] = b;
        });
    }
    let domain = CanonicCoset::new(log_size).circle_domain();
    trace
        .into_iter()
        .map(|eval| CircleEvaluation::<SimdBackend, _, BitReversedOrder>::new(domain, eval))
        .collect_vec()
}

pub fn save_secure_field_element(file: &mut File, q31_element: SecureField) -> Result<(), Error> {
    file.write_all(b"[\"")?;
    file.write_all(&q31_element.0.0.to_string().into_bytes())?;
    file.write_all(b"\",\"")?;
    file.write_all(&q31_element.0.1.to_string().into_bytes())?;
    file.write_all(b"\",\"")?;
    file.write_all(&q31_element.1.0.to_string().into_bytes())?;
    file.write_all(b"\",\"")?;
    file.write_all(&q31_element.1.1.to_string().into_bytes())?;
    file.write_all(b"\"]")?;
    Ok(())
}

pub fn pretty_save_poseidon_bls_proof(proof: &StarkProof<PoseidonBLSMerkleHasher>) -> Result<(),Error> {
    let mut file = File::create("proof.json")?;
    file.write_all(b"{\n")?;
    //commitments
    file.write_all(b"\t\"commitments\" : \n\t\t[")?;
    for i in 0..proof.commitments.len() {
        file.write_all(b"\"")?;
        file.write_all(&proof.commitments[i].to_string().into_bytes())?;
        file.write_all(b"\"")?;
        if proof.commitments.len() != i+1 {
            file.write_all(b",\n\t\t")?;
        }
    }
    file.write_all(b"],\n\n")?;
    // Sampled Values
    file.write_all(b"\t\"sampled_values_0\" : \n\t\t[")?;
    for i in 0..proof.commitment_scheme_proof.sampled_values.0[0].len() {
        save_secure_field_element(&mut file,proof.commitment_scheme_proof.sampled_values.0[0][i][0])?;
        if proof.commitment_scheme_proof.sampled_values.0[0].len() != i+1 {
            file.write_all(b",\n\t\t")?;
        }
    }
    file.write_all(b"],\n\n")?;
    file.write_all(b"\t\"sampled_values_1\" : \n\t\t[")?;
    for i in 0..proof.commitment_scheme_proof.sampled_values.0[1].len() {
        save_secure_field_element(&mut file,proof.commitment_scheme_proof.sampled_values.0[1][i][0])?;
        if proof.commitment_scheme_proof.sampled_values.0[1].len() != i+1 {
            file.write_all(b",\n\t\t")?;
        }
    }
    file.write_all(b"],\n\n")?;

    //decommitments
    file.write_all(b"\t\"decommitment_0\" : \n\t\t[")?;
    for i in 0..proof.commitment_scheme_proof.decommitments.0[0].hash_witness.len() {
        file.write_all(b"\"")?;
        file.write_all(&proof.commitment_scheme_proof.decommitments.0[0].hash_witness[i].to_string().into_bytes())?;
        file.write_all(b"\"")?;
        if proof.commitment_scheme_proof.decommitments.0[0].hash_witness.len() != i+1 {
            file.write_all(b",\n\t\t")?;
        }
    }
    file.write_all(b"],\n\n")?;
    file.write_all(b"\t\"decommitment_1\" : \n\t\t[")?;
    for i in 0..proof.commitment_scheme_proof.decommitments.0[1].hash_witness.len() {
        file.write_all(b"\"")?;
        file.write_all(&proof.commitment_scheme_proof.decommitments.0[1].hash_witness[i].to_string().into_bytes())?;
        file.write_all(b"\"")?;
        if proof.commitment_scheme_proof.decommitments.0[1].hash_witness.len() != i+1 {
            file.write_all(b",\n\t\t")?;
        }
    }
    file.write_all(b"],\n\n")?;

    //Queried_values
    file.write_all(b"\t\"queried_values_0\" : \n\t\t[")?;
    for i in 0..proof.commitment_scheme_proof.queried_values.0[0].len() {
        file.write_all(b"[")?;
        for j in 0..6 {
            file.write_all(b"\"")?;
            file.write_all(&proof.commitment_scheme_proof.queried_values.0[0][i][j].to_string().into_bytes())?;
            file.write_all(b"\"")?;
            if j != 5 {
                file.write_all(b",")?;
            } else { file.write_all(b"]")?; }
        }
        if proof.commitment_scheme_proof.queried_values.0[0].len() != i+1 {
            file.write_all(b",\n\t\t")?;
        }
    }
    file.write_all(b"],\n\n")?;
    file.write_all(b"\t\"queried_values_1\" : [")?;
    for i in 0..proof.commitment_scheme_proof.queried_values.0[1].len() {
        file.write_all(b"[")?;
        for j in 0..6 {
            file.write_all(b"\"")?;
            file.write_all(&proof.commitment_scheme_proof.queried_values.0[1][i][j].to_string().into_bytes())?;
            file.write_all(b"\"")?;
            if j != 5 {
                file.write_all(b",")?;
            } else { file.write_all(b"]")?; }
        }
        if proof.commitment_scheme_proof.queried_values.0[1].len() != i+1 {
            file.write_all(b",\n\t\t")?;
        }
    }
    file.write_all(b"],\n\n")?;

    //proof of work
    file.write_all(b"\t\"proof of work\" : \"")?;
    file.write_all(&proof.commitment_scheme_proof.proof_of_work.to_string().into_bytes())?;
    file.write_all(b"\",\n\n")?;

    //last FRI layer coeffs
    file.write_all(b"\t\"coeffs\" : ")?;
    save_secure_field_element(&mut file,proof.commitment_scheme_proof.fri_proof.last_layer_poly.coeffs[0] )?;
    file.write_all(b",\n\n")?;

    //intermediate FRI layers
    for i in 0..6 {
        //commitment
        file.write_all(b"\t\"inner_commitment_")?;
        file.write_all(&i.to_string().into_bytes())?;
        file.write_all(b"\" : \"")?;
        file.write_all(&proof.commitment_scheme_proof.fri_proof.inner_layers[i].commitment.0.to_string().into_bytes())?;
        file.write_all(b"\",\n\n")?;

        //decommitment
        file.write_all(b"\t\"inner_decommitment_")?;
        file.write_all(&i.to_string().into_bytes())?;
        file.write_all(b"\" : \n\t\t[")?;
        for j in 0..proof.commitment_scheme_proof.fri_proof.inner_layers[i].decommitment.hash_witness.len() {
            file.write_all(b"\"")?;
            file.write_all(&proof.commitment_scheme_proof.fri_proof.inner_layers[i].decommitment.hash_witness[j].to_string().into_bytes())?;
            file.write_all(b"\"")?;
            if proof.commitment_scheme_proof.fri_proof.inner_layers[i].decommitment.hash_witness.len() != j+1 {
                file.write_all(b",\n\t\t")?;
            }
        }
        file.write_all(b"],\n\n")?;

        //evals_subset
        file.write_all(b"\t\"inner_evals_subset_")?;
        file.write_all(&i.to_string().into_bytes())?;
        file.write_all(b"\" : \n\t\t[")?;
        for j in 0..proof.commitment_scheme_proof.fri_proof.inner_layers[i].evals_subset.len() {
            save_secure_field_element(&mut file,proof.commitment_scheme_proof.fri_proof.inner_layers[i].evals_subset[j])?;
            if proof.commitment_scheme_proof.fri_proof.inner_layers[i].evals_subset.len() != j+1 {
                file.write_all(b",\n\t\t")?;
            }
        }
        if i != 5 {
            file.write_all(b"],\n\n")?;
        } else {
            file.write_all(b"]\n}")?;
        }
    }
    Ok(())
}

pub fn compressed_save_poseidon_bls_proof(proof: &StarkProof<PoseidonBLSMerkleHasher>) -> Result<(),Error> {
    let mut file = File::create("proof.json")?;
    file.write_all(b"{")?;
    //commitments
    file.write_all(b"\"commitments\":[")?;
    for i in 0..proof.commitments.len() {
        file.write_all(b"\"")?;
        file.write_all(&proof.commitments[i].to_string().into_bytes())?;
        file.write_all(b"\"")?;
        if proof.commitments.len() != i+1 {
            file.write_all(b",")?;
        }
    }
    file.write_all(b"],")?;
    // Sampled Values
    file.write_all(b"\"sampled_values_0\":[")?;
    for i in 0..proof.commitment_scheme_proof.sampled_values.0[0].len() {
        save_secure_field_element(&mut file,proof.commitment_scheme_proof.sampled_values.0[0][i][0])?;
        if proof.commitment_scheme_proof.sampled_values.0[0].len() != i+1 {
            file.write_all(b",")?;
        }
    }
    file.write_all(b"],")?;
    file.write_all(b"\"sampled_values_1\":[")?;
    for i in 0..proof.commitment_scheme_proof.sampled_values.0[1].len() {
        save_secure_field_element(&mut file,proof.commitment_scheme_proof.sampled_values.0[1][i][0])?;
        if proof.commitment_scheme_proof.sampled_values.0[1].len() != i+1 {
            file.write_all(b",")?;
        }
    }
    file.write_all(b"],")?;

    //decommitments
    file.write_all(b"\"decommitment_0\":[")?;
    for i in 0..proof.commitment_scheme_proof.decommitments.0[0].hash_witness.len() {
        file.write_all(b"\"")?;
        file.write_all(&proof.commitment_scheme_proof.decommitments.0[0].hash_witness[i].to_string().into_bytes())?;
        file.write_all(b"\"")?;
        if proof.commitment_scheme_proof.decommitments.0[0].hash_witness.len() != i+1 {
            file.write_all(b",")?;
        }
    }
    file.write_all(b"],")?;
    file.write_all(b"\"decommitment_1\":[")?;
    for i in 0..proof.commitment_scheme_proof.decommitments.0[1].hash_witness.len() {
        file.write_all(b"\"")?;
        file.write_all(&proof.commitment_scheme_proof.decommitments.0[1].hash_witness[i].to_string().into_bytes())?;
        file.write_all(b"\"")?;
        if proof.commitment_scheme_proof.decommitments.0[1].hash_witness.len() != i+1 {
            file.write_all(b",")?;
        }
    }
    file.write_all(b"],")?;

    //Queried_values
    file.write_all(b"\"queried_values_0\":[")?;
    for i in 0..proof.commitment_scheme_proof.queried_values.0[0].len() {
        file.write_all(b"[")?;
        for j in 0..6 {
            file.write_all(b"\"")?;
            file.write_all(&proof.commitment_scheme_proof.queried_values.0[0][i][j].to_string().into_bytes())?;
            file.write_all(b"\"")?;
            if j != 5 {
                file.write_all(b",")?;
            } else { file.write_all(b"]")?; }
        }
        if proof.commitment_scheme_proof.queried_values.0[0].len() != i+1 {
            file.write_all(b",")?;
        }
    }
    file.write_all(b"],")?;
    file.write_all(b"\"queried_values_1\":[")?;
    for i in 0..proof.commitment_scheme_proof.queried_values.0[1].len() {
        file.write_all(b"[")?;
        for j in 0..6 {
            file.write_all(b"\"")?;
            file.write_all(&proof.commitment_scheme_proof.queried_values.0[1][i][j].to_string().into_bytes())?;
            file.write_all(b"\"")?;
            if j != 5 {
                file.write_all(b",")?;
            } else { file.write_all(b"]")?; }
        }
        if proof.commitment_scheme_proof.queried_values.0[1].len() != i+1 {
            file.write_all(b",")?;
        }
    }
    file.write_all(b"],")?;

    //proof of work
    file.write_all(b"\"proof of work\":\"")?;
    file.write_all(&proof.commitment_scheme_proof.proof_of_work.to_string().into_bytes())?;
    file.write_all(b"\",")?;

    //last FRI layer coeffs
    file.write_all(b"\"coeffs\":")?;
    save_secure_field_element(&mut file,proof.commitment_scheme_proof.fri_proof.last_layer_poly.coeffs[0] )?;
    file.write_all(b",")?;

    //intermediate FRI layers
    for i in 0..6 {
        //commitment
        file.write_all(b"\"inner_commitment_")?;
        file.write_all(&i.to_string().into_bytes())?;
        file.write_all(b"\":\"")?;
        file.write_all(&proof.commitment_scheme_proof.fri_proof.inner_layers[i].commitment.0.to_string().into_bytes())?;
        file.write_all(b"\",")?;

        //decommitment
        file.write_all(b"\"inner_decommitment_")?;
        file.write_all(&i.to_string().into_bytes())?;
        file.write_all(b"\":[")?;
        for j in 0..proof.commitment_scheme_proof.fri_proof.inner_layers[i].decommitment.hash_witness.len() {
            file.write_all(b"\"")?;
            file.write_all(&proof.commitment_scheme_proof.fri_proof.inner_layers[i].decommitment.hash_witness[j].to_string().into_bytes())?;
            file.write_all(b"\"")?;
            if proof.commitment_scheme_proof.fri_proof.inner_layers[i].decommitment.hash_witness.len() != j+1 {
                file.write_all(b",")?;
            }
        }
        file.write_all(b"],")?;

        //evals_subset
        file.write_all(b"\"inner_evals_subset_")?;
        file.write_all(&i.to_string().into_bytes())?;
        file.write_all(b"\":[")?;
        for j in 0..proof.commitment_scheme_proof.fri_proof.inner_layers[i].evals_subset.len() {
            save_secure_field_element(&mut file,proof.commitment_scheme_proof.fri_proof.inner_layers[i].evals_subset[j])?;
            if proof.commitment_scheme_proof.fri_proof.inner_layers[i].evals_subset.len() != j+1 {
                file.write_all(b",")?;
            }
        }
        if i != 5 {
            file.write_all(b"],")?;
        } else {
            file.write_all(b"]}")?;
        }
    }




    Ok(())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use num_traits::One;

    use super::{pretty_save_poseidon_bls_proof, WideFibonacciEval};
    use crate::constraint_framework::{
        assert_constraints, AssertEvaluator, FrameworkEval, TraceLocationAllocator,
    };
    use crate::core::air::Component;
    use crate::core::backend::simd::m31::{PackedBaseField, LOG_N_LANES};
    use crate::core::backend::simd::SimdBackend;
    use crate::core::backend::Column;
    use crate::core::channel::Blake2sChannel;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::core::channel::Poseidon252Channel;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::core::channel::PoseidonBLSChannel;
    use crate::core::fields::m31::BaseField;
    use crate::core::pcs::{CommitmentSchemeProver, CommitmentSchemeVerifier, PcsConfig, TreeVec};
    use crate::core::poly::circle::{CanonicCoset, CircleEvaluation, PolyOps};
    use crate::core::poly::BitReversedOrder;
    use crate::core::prover::{prove, verify};
    use crate::core::vcs::blake2_merkle::Blake2sMerkleChannel;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::core::vcs::poseidon252_merkle::Poseidon252MerkleChannel;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::core::vcs::poseidon_bls_merkle::PoseidonBLSMerkleChannel;
    use crate::core::ColumnVec;
    use crate::core::fields::qm31::QM31;
    use crate::examples::wide_fibonacci::{generate_trace, FibInput, WideFibonacciComponent};

    const FIB_SEQUENCE_LENGTH: usize = 100;

    fn generate_test_trace(
        log_n_instances: u32,
    ) -> ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
        let inputs = (0..(1 << (log_n_instances - LOG_N_LANES)))
            .map(|i| FibInput {
                a: PackedBaseField::one(),
                b: PackedBaseField::from_array(std::array::from_fn(|j| {
                    BaseField::from_u32_unchecked((i * 16 + j) as u32)
                })),
            })
            .collect_vec();
        generate_trace::<FIB_SEQUENCE_LENGTH>(log_n_instances, &inputs)
    }

    fn fibonacci_constraint_evaluator<const N: u32>(eval: AssertEvaluator<'_>) {
        WideFibonacciEval::<FIB_SEQUENCE_LENGTH> { log_n_rows: N }.evaluate(eval);
    }

    #[test]
    fn test_wide_fibonacci_constraints() {
        const LOG_N_INSTANCES: u32 = 6;
        let traces = TreeVec::new(vec![generate_test_trace(LOG_N_INSTANCES)]);
        let trace_polys =
            traces.map(|trace| trace.into_iter().map(|c| c.interpolate()).collect_vec());

        assert_constraints(
            &trace_polys,
            CanonicCoset::new(LOG_N_INSTANCES),
            fibonacci_constraint_evaluator::<LOG_N_INSTANCES>,
        );
    }

    #[test]
    #[should_panic]
    fn test_wide_fibonacci_constraints_fails() {
        const LOG_N_INSTANCES: u32 = 6;

        let mut trace = generate_test_trace(LOG_N_INSTANCES);
        // Modify the trace such that a constraint fail.
        trace[17].values.set(2, BaseField::one());
        let traces = TreeVec::new(vec![trace]);
        let trace_polys =
            traces.map(|trace| trace.into_iter().map(|c| c.interpolate()).collect_vec());

        assert_constraints(
            &trace_polys,
            CanonicCoset::new(LOG_N_INSTANCES),
            fibonacci_constraint_evaluator::<LOG_N_INSTANCES>,
        );
    }

    #[test_log::test]
    fn test_wide_fib_prove() {
        const LOG_N_INSTANCES: u32 = 6;
        let config = PcsConfig::default();
        // Precompute twiddles.
        let twiddles = SimdBackend::precompute_twiddles(
            CanonicCoset::new(LOG_N_INSTANCES + 1 + config.fri_config.log_blowup_factor)
                .circle_domain()
                .half_coset,
        );

        // Setup protocol.
        let prover_channel = &mut Blake2sChannel::default();
        let commitment_scheme =
            &mut CommitmentSchemeProver::<SimdBackend, Blake2sMerkleChannel>::new(
                config, &twiddles,
            );

        // Trace.
        let trace = generate_test_trace(LOG_N_INSTANCES);
        let mut tree_builder = commitment_scheme.tree_builder();
        tree_builder.extend_evals(trace);
        tree_builder.commit(prover_channel);

        // Prove constraints.
        let component = WideFibonacciComponent::new(
            &mut TraceLocationAllocator::default(),
            WideFibonacciEval::<FIB_SEQUENCE_LENGTH> {
                log_n_rows: LOG_N_INSTANCES,
            },
        );

        let proof = prove::<SimdBackend, Blake2sMerkleChannel>(
            &[&component],
            prover_channel,
            commitment_scheme,
        )
        .unwrap();

        // Verify.
        let verifier_channel = &mut Blake2sChannel::default();
        let commitment_scheme = &mut CommitmentSchemeVerifier::<Blake2sMerkleChannel>::new(config);

        // Retrieve the expected column sizes in each commitment interaction, from the AIR.
        let sizes = component.trace_log_degree_bounds();
        commitment_scheme.commit(proof.commitments[0], &sizes[0], verifier_channel);
        verify(&[&component], verifier_channel, commitment_scheme, proof).unwrap();
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_wide_fib_prove_with_poseidon() {
        const LOG_N_INSTANCES: u32 = 6;

        let config = PcsConfig::default();
        // Precompute twiddles.
        let twiddles = SimdBackend::precompute_twiddles(
            CanonicCoset::new(LOG_N_INSTANCES + 1 + config.fri_config.log_blowup_factor)
                .circle_domain()
                .half_coset,
        );

        // Setup protocol.
        let prover_channel = &mut Poseidon252Channel::default();
        let commitment_scheme =
            &mut CommitmentSchemeProver::<SimdBackend, Poseidon252MerkleChannel>::new(
                config, &twiddles,
            );

        // Trace.
        let trace = generate_test_trace(LOG_N_INSTANCES);
        //Initialize the parameters of the trace
        let mut tree_builder = commitment_scheme.tree_builder();
        // Interpolation of the columns
        tree_builder.extend_evals(trace);
        // Compute the evaluations of the polynomials, build a Merkle tree of the evaluation and
        // update the channel (Fiat-Shamir) with the root of the tree (Transcript <-- root)
        tree_builder.commit(prover_channel);

        // Prove constraints.
        let component = WideFibonacciComponent::new(
            &mut TraceLocationAllocator::default(),
            WideFibonacciEval::<FIB_SEQUENCE_LENGTH> {
                log_n_rows: LOG_N_INSTANCES,
            },
        );
        let proof = prove::<SimdBackend, Poseidon252MerkleChannel>(
            &[&component],
            prover_channel,
            commitment_scheme,
        )
        .unwrap();

        // Verify.
        let verifier_channel = &mut Poseidon252Channel::default();
        let commitment_scheme =
            &mut CommitmentSchemeVerifier::<Poseidon252MerkleChannel>::new(config);

        // Retrieve the expected column sizes in each commitment interaction, from the AIR.
        let sizes = component.trace_log_degree_bounds();
        commitment_scheme.commit(proof.commitments[0], &sizes[0], verifier_channel);
        verify(&[&component], verifier_channel, commitment_scheme, proof).unwrap();
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_wide_fib_prove_with_poseidon_bls() {

        const LOG_N_INSTANCES: u32 = 6;

        let config = PcsConfig::default();
        // Precompute twiddles.
        let twiddles = SimdBackend::precompute_twiddles(
            CanonicCoset::new(LOG_N_INSTANCES + 1 + config.fri_config.log_blowup_factor)
                .circle_domain()
                .half_coset,
        );

        // Setup protocol.
        let prover_channel = &mut PoseidonBLSChannel::default();
        let commitment_scheme =
            &mut CommitmentSchemeProver::<SimdBackend, PoseidonBLSMerkleChannel>::new(
                config, &twiddles,
            );

        // Trace.
        let trace = generate_test_trace(LOG_N_INSTANCES);
        //Initialize the parameters of the trace
        let mut tree_builder = commitment_scheme.tree_builder();
        // Interpolation of the columns
        tree_builder.extend_evals(trace);
        // Compute the evaluations of the polynomials, build a Merkle tree of the evaluation and
        // update the channel (Fiat-Shamir) with the root of the tree (Transcript <-- root)
        tree_builder.commit(prover_channel);

        // Prove constraints.
        let component = WideFibonacciComponent::new(
            &mut TraceLocationAllocator::default(),
            WideFibonacciEval::<FIB_SEQUENCE_LENGTH> {
                log_n_rows: LOG_N_INSTANCES,
            },
        );
        let proof = prove::<SimdBackend, PoseidonBLSMerkleChannel>(
            &[&component],
            prover_channel,
            commitment_scheme,
        )
            .unwrap();
        _ = pretty_save_poseidon_bls_proof(&proof);
        println!(" 0 1 0 0 = {:?}",QM31::from_u32_unchecked(0,1,0,0));

        // Verify.
        let verifier_channel = &mut PoseidonBLSChannel::default();
        let commitment_scheme =
            &mut CommitmentSchemeVerifier::<PoseidonBLSMerkleChannel>::new(config);

        // Retrieve the expected column sizes in each commitment interaction, from the AIR.
        let sizes = component.trace_log_degree_bounds();
        commitment_scheme.commit(proof.commitments[0], &sizes[0], verifier_channel);
        verify(&[&component], verifier_channel, commitment_scheme, proof).unwrap();
    }
}

