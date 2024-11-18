use cl::{zones::*, Output};
use ledger_proof_statements::{bundle::*, constraint::*, ledger::*, pact::PactPublic, ptx::*};
use risc0_zkvm::{guest::env, serde};

fn main() {
    let LedgerProofPrivate {
        mut ledger,
        id,
        txs,
    } = env::read();

    let cm_root = ledger.cm_root();

    let mut cross_in = vec![];
    let mut cross_out = vec![];

    for tx in txs {
        match tx {
            ZoneTx::LocalTx { bundle, ptxs } => {
                ledger = process_bundle(ledger, ptxs, bundle, cm_root);
            }
            ZoneTx::Pact(pact) => {
                let (new_ledger, consumed_commits, produced_commits) =
                    process_pact(ledger, pact, id, cm_root);
                ledger = new_ledger;
                cross_in.extend(consumed_commits);
                cross_out.extend(produced_commits);
            }
        }
    }

    env::commit(&LedgerProofPublic {
        ledger: ledger.commit(),
        id,
        cross_in,
        cross_out,
    });
}

fn process_bundle(
    mut ledger: LedgerWitness,
    ptxs: Vec<PtxPublic>,
    bundle_proof: BundlePublic,
    cm_root: [u8; 32],
) -> LedgerWitness {
    assert_eq!(
        ptxs.iter().map(|ptx| ptx.ptx.balance).collect::<Vec<_>>(),
        bundle_proof.balances
    );
    // verify bundle is balanced
    env::verify(
        nomos_cl_risc0_proofs::BUNDLE_ID,
        &serde::to_vec(&bundle_proof).unwrap(),
    )
    .unwrap();

    for ptx in ptxs {
        ledger = process_ptx(ledger, ptx, cm_root);
    }

    ledger
}

fn process_ptx(mut ledger: LedgerWitness, ptx: PtxPublic, cm_root: [u8; 32]) -> LedgerWitness {
    env::verify(nomos_cl_risc0_proofs::PTX_ID, &serde::to_vec(&ptx).unwrap()).unwrap();
    assert_eq!(ptx.cm_root, cm_root);

    let ptx = ptx.ptx;

    for input in &ptx.inputs {
        assert!(!ledger.nullifiers.contains(&input.nullifier));
        ledger.nullifiers.push(input.nullifier);

        env::verify(
            input.constraint.0,
            &serde::to_vec(&ConstraintPublic {
                ptx_root: ptx.root(),
                nf: input.nullifier,
            })
            .unwrap(),
        )
        .unwrap();
    }

    for output in &ptx.outputs {
        ledger.commitments.push(output.note_comm);
    }

    ledger
}

fn process_pact(
    mut ledger: LedgerWitness,
    pact: PactPublic,
    id: ZoneId,
    cm_root: [u8; 32],
) -> (LedgerWitness, Vec<Output>, Vec<Output>) {
    let mut cross_in = vec![];
    let mut cross_out = vec![];

    env::verify(
        nomos_cl_risc0_proofs::PACT_ID,
        &serde::to_vec(&pact).unwrap(),
    )
    .unwrap();

    let pact_cm_root = pact.cm_root;
    let pact = pact.pact;

    if cm_root != pact_cm_root {
        // zone is the receiver of the transfer
        for (comm, zone) in pact.tx.outputs.iter().zip(&pact.to) {
            if *zone == id {
                cross_in.push(*comm);
                ledger.commitments.push(comm.note_comm);
            }
        }
    } else {
        // zone is the sender of the transfer
        // proof of non-membership
        for input in &pact.tx.inputs {
            assert!(!ledger.nullifiers.contains(&input.nullifier));
            ledger.nullifiers.push(input.nullifier);

            env::verify(
                input.constraint.0,
                &serde::to_vec(&ConstraintPublic {
                    ptx_root: pact.tx.root(),
                    nf: input.nullifier,
                })
                .unwrap(),
            )
            .unwrap();
        }

        for (output, to) in pact.tx.outputs.iter().zip(&pact.to) {
            if *to == id {
                ledger.commitments.push(output.note_comm);
            } else {
                cross_out.push(*output);
            }
        }
    }

    (ledger, cross_in, cross_out)
}
