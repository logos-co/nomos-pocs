use proof_statements::input::{InputPrivate, InputPublic};

use crate::error::{Error, Result};

const MAX_NOTE_COMMS: usize = 2usize.pow(8);

#[derive(Debug, Clone)]
pub struct ProvedInput {
    pub input: InputPublic,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedInput {
    pub fn prove(
        input: &cl::InputWitness,
        note_commitments: &[cl::NoteCommitment],
    ) -> Result<Self> {
        let output_cm = input.note_commitment();

        let cm_leaves = note_commitment_leaves(note_commitments);
        let cm_idx = note_commitments
            .iter()
            .position(|c| c == &output_cm)
            .unwrap();
        let cm_path = cl::merkle::path(cm_leaves, cm_idx);

        let secrets = InputPrivate {
            input: *input,
            cm_path,
        };

        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&secrets)
            .unwrap()
            .build()
            .unwrap();

        // Obtain the default prover.
        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        // Proof information by proving the specified ELF binary.
        // This struct contains the receipt along with statistics about execution of the guest
        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_cl_risc0_proofs::INPUT_ELF, &opts)
            .map_err(|_| Error::Risc0ProofFailed)?;

        println!(
            "STARK 'input' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );
        // extract the receipt.
        let receipt = prove_info.receipt;

        Ok(Self {
            input: InputPublic {
                cm_root: cl::merkle::root(cm_leaves),
                input: input.commit(),
            },
            risc0_receipt: receipt,
        })
    }

    pub fn public(&self) -> Result<InputPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(proved_public_inputs) = self.public() else {
            return false;
        };

        self.input == proved_public_inputs
            && self
                .risc0_receipt
                .verify(nomos_cl_risc0_proofs::INPUT_ID)
                .is_ok()
    }
}

fn note_commitment_leaves(note_commitments: &[cl::NoteCommitment]) -> [[u8; 32]; MAX_NOTE_COMMS] {
    let note_comm_bytes = Vec::from_iter(note_commitments.iter().map(|c| c.as_bytes().to_vec()));
    let cm_leaves = cl::merkle::padded_leaves::<MAX_NOTE_COMMS>(&note_comm_bytes);
    cm_leaves
}

#[cfg(test)]
mod test {
    use rand::thread_rng;

    use super::*;

    #[test]
    fn test_input_prover() {
        let mut rng = thread_rng();

        let input = cl::InputWitness {
            note: cl::NoteWitness::basic(32, "NMO"),
            balance_blinding: cl::BalanceWitness::random(&mut rng),
            nf_sk: cl::NullifierSecret::random(&mut rng),
            nonce: cl::NullifierNonce::random(&mut rng),
        };

        let notes = vec![input.note_commitment()];

        let mut proved_input = ProvedInput::prove(&input, &notes).unwrap();

        let expected_public_inputs = InputPublic {
            cm_root: cl::merkle::root(note_commitment_leaves(&notes)),
            input: input.commit(),
        };

        assert_eq!(proved_input.input, expected_public_inputs);
        assert!(proved_input.verify());

        let wrong_public_inputs = [
            InputPublic {
                cm_root: cl::merkle::root([cl::merkle::leaf(b"bad_root")]),
                ..expected_public_inputs
            },
            InputPublic {
                input: cl::Input {
                    nullifier: cl::Nullifier::new(
                        cl::NullifierSecret::random(&mut rng),
                        cl::NullifierNonce::random(&mut rng),
                    ),
                    ..expected_public_inputs.input
                },
                ..expected_public_inputs
            },
            InputPublic {
                input: cl::Input {
                    death_cm: cl::note::death_commitment(b"wrong death vk"),
                    ..expected_public_inputs.input
                },
                ..expected_public_inputs
            },
            InputPublic {
                input: cl::Input {
                    balance: cl::BalanceWitness::random(&mut rng)
                        .commit(&cl::NoteWitness::basic(32, "NMO")),
                    ..expected_public_inputs.input
                },
                ..expected_public_inputs
            },
        ];

        for wrong_input in wrong_public_inputs {
            proved_input.input = wrong_input;
            assert!(!proved_input.verify());
        }
    }

    // ----- The following tests still need to be built. -----
    // #[test]
    // fn test_input_ptx_coupling() {
    //     let mut rng = rand::thread_rng();

    //     let note = cl::NoteWitness::new(10, "NMO", [0u8; 32], &mut rng);
    //     let nf_sk = cl::NullifierSecret::random(&mut rng);
    //     let nonce = cl::NullifierNonce::random(&mut rng);

    //     let witness = cl::InputWitness { note, nf_sk, nonce };

    //     let input = witness.commit();

    //     let ptx_root = cl::PtxRoot::random(&mut rng);
    //     let proof = input.prove(&witness, ptx_root, vec![]).unwrap();

    //     assert!(input.verify(ptx_root, &proof));

    //     // The same input proof can not be used in another partial transaction.
    //     let another_ptx_root = cl::PtxRoot::random(&mut rng);
    //     assert!(!input.verify(another_ptx_root, &proof));
    // }
}
