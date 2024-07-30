use crate::error::{Error, Result};

pub struct ProvedOutput {
    pub output: cl::Output,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedOutput {
    pub fn prove(witness: &cl::OutputWitness) -> Result<Self> {
        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&witness)
            .unwrap()
            .build()
            .unwrap();

        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_cl_risc0_proofs::OUTPUT_ELF, &opts)
            .map_err(|_| Error::Risc0ProofFailed)?;

        println!(
            "STARK 'output' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        let receipt = prove_info.receipt;

        Ok(Self {
            output: witness.commit(),
            risc0_receipt: receipt,
        })
    }

    pub fn public(&self) -> Result<cl::Output> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(output_commitments) = self.public() else {
            return false;
        };

        self.output == output_commitments
            && self
                .risc0_receipt
                .verify(nomos_cl_risc0_proofs::OUTPUT_ID)
                .is_ok()
    }
}

#[cfg(test)]
mod test {
    use rand::thread_rng;

    use super::*;

    #[test]
    fn test_output_prover() {
        let mut rng = thread_rng();

        let output = cl::OutputWitness {
            note: cl::NoteWitness::basic(32, "NMO"),
            balance_blinding: cl::BalanceWitness::random(&mut rng),
            nf_pk: cl::NullifierSecret::random(&mut rng).commit(),
            nonce: cl::NullifierNonce::random(&mut rng),
        };

        let mut proved_output = ProvedOutput::prove(&output).unwrap();

        let expected_output_cm = output.commit();

        assert_eq!(proved_output.output, expected_output_cm);
        assert!(proved_output.verify());

        let wrong_output_cms = [
            cl::Output {
                note_comm: cl::NoteWitness::basic(100, "NMO").commit(
                    cl::NullifierSecret::random(&mut rng).commit(),
                    cl::NullifierNonce::random(&mut rng),
                ),
                ..expected_output_cm
            },
            cl::Output {
                note_comm: cl::NoteWitness::basic(100, "NMO").commit(
                    cl::NullifierSecret::random(&mut rng).commit(),
                    cl::NullifierNonce::random(&mut rng),
                ),
                balance: cl::BalanceWitness::random(&mut rng)
                    .commit(&cl::NoteWitness::basic(100, "NMO")),
            },
        ];

        for wrong_output_cm in wrong_output_cms {
            proved_output.output = wrong_output_cm;
            assert!(!proved_output.verify());
        }
    }

    #[test]
    fn test_zero_output_is_rejected() {
        let mut rng = thread_rng();

        let output = cl::OutputWitness::random(
            cl::NoteWitness::basic(0, "NMO"),
            cl::NullifierSecret::random(&mut rng).commit(),
            &mut rng,
        );

        assert!(ProvedOutput::prove(&output).is_err());
    }
}
