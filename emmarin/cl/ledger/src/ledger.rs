use ledger_proof_statements::ledger::{LedgerProofPrivate, LedgerProofPublic, ZoneTx};

use crate::{
    bundle::ProvedBundle,
    constraint::ConstraintProof,
    error::{Error, Result},
    pact::ProvedPact,
    partial_tx::ProvedPartialTx,
};
use cl::zone_layer::{LedgerWitness, ZoneId};

pub struct ProvedLedgerTransition {
    pub public: LedgerProofPublic,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

pub enum ProvedZoneTx {
    LocalTx {
        bundle: ProvedBundle,
        ptxs: Vec<ProvedPartialTx>,
    },
    Pact(ProvedPact),
}

impl ProvedZoneTx {
    fn to_public(&self) -> ZoneTx {
        match self {
            Self::LocalTx { ptxs, bundle } => ZoneTx::LocalTx {
                ptxs: ptxs.iter().map(|p| p.public().unwrap()).collect(),
                bundle: bundle.public().unwrap(),
            },
            Self::Pact(pact) => ZoneTx::Pact(pact.public().unwrap()),
        }
    }

    fn proofs(&self) -> Vec<risc0_zkvm::Receipt> {
        match self {
            Self::LocalTx { ptxs, bundle } => {
                let mut proofs = vec![bundle.risc0_receipt.clone()];
                proofs.extend(ptxs.iter().map(|p| p.risc0_receipt.clone()));
                proofs
            }
            Self::Pact(pact) => vec![pact.risc0_receipt.clone()],
        }
    }
}

impl ProvedLedgerTransition {
    pub fn prove(
        ledger: LedgerWitness,
        zone_id: ZoneId,
        ptxs: Vec<ProvedZoneTx>,
        constraints: Vec<ConstraintProof>,
    ) -> Result<Self> {
        let witness = LedgerProofPrivate {
            txs: ptxs.iter().map(|p| p.to_public()).collect(),
            ledger,
            id: zone_id,
        };

        let mut env = risc0_zkvm::ExecutorEnv::builder();

        for ptx in ptxs {
            for proof in ptx.proofs() {
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

    pub fn public(&self) -> Result<LedgerProofPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        // let Ok(proved_ptx_inputs) = self.public() else {
        //     return false;
        // };
        // let expected_ptx_inputs = PtxPublic {
        //     ptx: self.ptx.clone(),
        //     cm_root: self.cm_root,
        //     from: self.from,
        //     to: self.to,
        // };
        // if expected_ptx_inputs != proved_ptx_inputs {
        //     return false;
        // }

        // let ptx_root = self.ptx.root();

        self.risc0_receipt
            .verify(ledger_validity_proof::LEDGER_ID)
            .is_ok()
    }
}
