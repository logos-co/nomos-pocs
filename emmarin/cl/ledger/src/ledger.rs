use ledger_proof_statements::ledger::{
    LedgerBundleWitness, LedgerProofPrivate, LedgerProofPublic, LedgerPtxWitness,
};

use crate::{
    balance::ProvedBalance,
    constraint::ConstraintProof,
    error::{Error, Result},
    partial_tx::ProvedPartialTx,
};
use cl::zone_layer::{ledger::LedgerState, notes::ZoneId};

#[derive(Debug, Clone)]
pub struct ProvedLedgerTransition {
    pub public: LedgerProofPublic,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

// TODO: find a better name
#[derive(Debug, Clone)]
pub struct ProvedBundle {
    pub balance: ProvedBalance,
    pub ptxs: Vec<ProvedPartialTx>,
}

impl ProvedBundle {
    fn proofs(&self) -> Vec<risc0_zkvm::Receipt> {
        let mut proofs = vec![self.balance.risc0_receipt.clone()];
        proofs.extend(self.ptxs.iter().map(|p| p.risc0_receipt.clone()));
        proofs
    }
}

impl ProvedLedgerTransition {
    pub fn prove(
        mut ledger: LedgerState,
        zone_id: ZoneId,
        bundles: Vec<ProvedBundle>,
        constraints: Vec<ConstraintProof>,
    ) -> Result<Self> {
        let mut witness = LedgerProofPrivate {
            bundles: Vec::new(),
            ledger: ledger.to_witness(),
            id: zone_id,
        };

        // prepare the sparse merkle tree nullifier proofs
        for bundle in &bundles {
            let mut partials = Vec::new();

            for ptx in &bundle.ptxs {
                let mut nf_proofs = Vec::new();

                for input in &ptx.public.ptx.inputs {
                    let nf_proof = ledger.add_nullifier(input.nullifier);
                    nf_proofs.push(nf_proof);
                }

                partials.push(LedgerPtxWitness {
                    ptx: ptx.public.clone(),
                    nf_proofs,
                });
            }

            witness.bundles.push(LedgerBundleWitness { partials })
        }

        let mut env = risc0_zkvm::ExecutorEnv::builder();

        for bundle in bundles {
            for proof in bundle.proofs() {
                env.add_assumption(proof);
            }
        }
        for covenant in constraints {
            env.add_assumption(covenant.risc0_receipt);
        }
        let env = env.write(&witness).unwrap().build().unwrap();

        // Obtain the default prover.
        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        // Proof information by proving the specified ELF binary.
        // This struct contains the receipt along with statistics about execution of the guest
        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, ledger_validity_proof::LEDGER_ELF, &opts)
            .map_err(|e| {
                eprintln!("{e}");
                Error::Risc0ProofFailed
            })?;

        println!(
            "STARK 'ledger' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        Ok(Self {
            public: prove_info
                .receipt
                .journal
                .decode::<LedgerProofPublic>()
                .unwrap(),
            risc0_receipt: prove_info.receipt,
        })
    }

    pub fn verify(&self) -> bool {
        self.risc0_receipt
            .verify(ledger_validity_proof::LEDGER_ID)
            .is_ok()
    }
}
