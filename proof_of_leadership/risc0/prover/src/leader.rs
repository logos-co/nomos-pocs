use crate::error::Result;

use curve25519_dalek::Scalar;

use proof_statements::proof_of_leadership::{LeaderPrivate, LeaderPublic};

const MAX_NOTE_COMMS: usize = 2usize.pow(8);


pub struct ProvedLeader {
    pub leader: LeaderPublic,
    pub risc0_receipt: risc0_zkvm::Receipt,
}


impl ProvedLeader {
    pub fn prove(input: &cl::InputWitness, epoch_nonce: [u8;32], slot: u64, active_slot_coefficient: f64, total_stake: u64, note_commitments: &[cl::NoteCommitment]) -> Self {
        let note_cm = input.note_commitment();
        let cm_leaves = note_commitment_leaves(note_commitments);
        let cm_idx = note_commitments
            .iter()
            .position(|c| c == &note_cm)
            .unwrap();
        let note_cm_path = cl::merkle::path(cm_leaves, cm_idx);
	let cm_root = cl::merkle::root(cm_leaves);

	let leader_private = LeaderPrivate {
	    input: *input,
	    input_cm_path: note_cm_path,
	};

	let leader_public = LeaderPublic::new(
	    cm_root,
	    epoch_nonce,
	    slot,
	    active_slot_coefficient,
	    total_stake,
	    input.nullifier(),
	    input.evolve_output(cl::BalanceWitness::new(Scalar::ZERO)).commit_note(),
	);

        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&leader_public)
            .unwrap()
            .write(&leader_private)
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
            .prove_with_opts(env, nomos_pol_risc0_proofs::PROOF_OF_LEADERSHIP_ELF, &opts)
            .unwrap();

        println!(
            "STARK prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );
        // extract the receipt.
        let receipt = prove_info.receipt;

        Self {
            leader: leader_public,
            risc0_receipt: receipt,
        }
    }

    pub fn public(&self) -> Result<LeaderPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(proved_public_inputs) = self.public() else {
            return false;
        };

        self.leader == proved_public_inputs
            && self
                .risc0_receipt
                .verify(nomos_pol_risc0_proofs::PROOF_OF_LEADERSHIP_ID)
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
    fn test_leader_prover() {
        let mut rng = thread_rng();

        let input = cl::InputWitness {
            note: cl::NoteWitness::basic(32, "NMO"),
            balance_blinding: cl::BalanceWitness::random(&mut rng),
            nf_sk: cl::NullifierSecret::random(&mut rng),
            nonce: cl::NullifierNonce::random(&mut rng),
        };

        let notes = vec![input.note_commitment()];
	let epoch_nonce = [0u8; 32];
	let slot = 0;
	let active_slot_coefficient = 0.05;
	let total_stake = 1000;

        let mut expected_public_inputs = LeaderPublic::new(
	    cl::merkle::root(note_commitment_leaves(&notes)),
	    epoch_nonce,
	    slot,
	    active_slot_coefficient,
	    total_stake,
	    input.nullifier(),
	    input.evolve_output(cl::BalanceWitness::new(Scalar::ZERO)).commit_note(),
	);

	while !expected_public_inputs.check_winning(&input) {
	    expected_public_inputs.slot += 1;
	}

	println!("slot={}", expected_public_inputs.slot);

        let proved_leader = ProvedLeader::prove(&input, expected_public_inputs.epoch_nonce, expected_public_inputs.slot, active_slot_coefficient, total_stake, &notes);


        assert_eq!(proved_leader.leader, expected_public_inputs);
        assert!(proved_leader.verify());

        // let wrong_public_inputs = [
        //     InputPublic {
        //         cm_root: cl::merkle::root([cl::merkle::leaf(b"bad_root")]),
        //         ..expected_public_inputs
        //     },
        //     InputPublic {
        //         input: cl::Input {
        //             nullifier: cl::Nullifier::new(
        //                 cl::NullifierSecret::random(&mut rng),
        //                 cl::NullifierNonce::random(&mut rng),
        //             ),
        //             ..expected_public_inputs.input
        //         },
        //         ..expected_public_inputs
        //     },
        //     InputPublic {
        //         input: cl::Input {
        //             death_cm: cl::note::death_commitment(b"wrong death vk"),
        //             ..expected_public_inputs.input
        //         },
        //         ..expected_public_inputs
        //     },
        //     InputPublic {
        //         input: cl::Input {
        //             balance: cl::BalanceWitness::random(&mut rng)
        //                 .commit(&cl::NoteWitness::basic(32, "NMO")),
        //             ..expected_public_inputs.input
        //         },
        //         ..expected_public_inputs
        //     },
        // ];

        // for wrong_input in wrong_public_inputs {
        //     proved_input.input = wrong_input;
        //     assert!(!proved_input.verify());
        // }
    }
}
