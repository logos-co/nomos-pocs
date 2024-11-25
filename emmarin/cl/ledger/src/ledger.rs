use ledger_proof_statements::{
    ledger::{LedgerProofPrivate, LedgerProofPublic},
    ptx::PtxPublic,
};

use crate::{
    balance::ProvedBalance,
    constraint::ConstraintProof,
    error::{Error, Result},
    partial_tx::ProvedPartialTx,
};
use cl::zone_layer::{ledger::LedgerWitness, notes::ZoneId};

#[derive(Debug, Clone)]
pub struct ProvedLedgerTransition {
    pub public: LedgerProofPublic,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

// TODO: find a better name
#[derive(Debug, Clone)]
pub struct ProvedBundle {
    pub bundle: ProvedBalance,
    pub ptxs: Vec<ProvedPartialTx>,
}

impl ProvedBundle {
    fn to_public(&self) -> Vec<PtxPublic> {
        self.ptxs.iter().map(|p| p.public.clone()).collect()
    }

    fn proofs(&self) -> Vec<risc0_zkvm::Receipt> {
        let mut proofs = vec![self.bundle.risc0_receipt.clone()];
        proofs.extend(self.ptxs.iter().map(|p| p.risc0_receipt.clone()));
        proofs
    }
}

impl ProvedLedgerTransition {
    pub fn prove(
        ledger: LedgerWitness,
        zone_id: ZoneId,
        bundles: Vec<ProvedBundle>,
        constraints: Vec<ConstraintProof>,
    ) -> Result<Self> {
        let witness = LedgerProofPrivate {
            bundles: bundles.iter().map(|p| p.to_public()).collect(),
            ledger,
            id: zone_id,
        };

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
