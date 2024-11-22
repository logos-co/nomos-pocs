use cl::{
    cl::Output,
    zone_layer::{ledger::LedgerWitness, notes::ZoneId},
};
use ledger_proof_statements::{
    bundle::BundlePublic,
    constraint::ConstraintPublic,
    ledger::{LedgerProofPrivate, LedgerProofPublic},
    ptx::PtxPublic,
};
use risc0_zkvm::{guest::env, serde};

fn main() {
    let LedgerProofPrivate {
        mut ledger,
        id,
        bundles,
    } = env::read();

    let old_ledger = ledger.commit();

    let cm_root = ledger.cm_root();

    let mut cross_in = vec![];
    let mut cross_out = vec![];

    for bundle in bundles {
        let bundle_public = BundlePublic {
            balances: bundle.iter().map(|ptx| ptx.ptx.balance).collect::<Vec<_>>(),
        };
        // verify bundle is balanced
        env::verify(
            nomos_cl_risc0_proofs::BUNDLE_ID,
            &serde::to_vec(&bundle_public).unwrap(),
        )
        .unwrap();

        for ptx in &bundle {
            let (new_ledger, consumed_commitments, produced_commitments) =
                process_ptx(ledger, ptx, id, cm_root);
            cross_in.extend(consumed_commitments);
            cross_out.extend(produced_commitments);
            ledger = new_ledger;
        }
    }

    env::commit(&LedgerProofPublic {
        old_ledger,
        ledger: ledger.commit(),
        id,
        cross_in,
        cross_out,
    });
}

fn process_ptx(
    mut ledger: LedgerWitness,
    ptx: &PtxPublic,
    zone_id: ZoneId,
    cm_root: [u8; 32],
) -> (LedgerWitness, Vec<Output>, Vec<Output>) {
    let mut cross_in = vec![];
    let mut cross_out = vec![];

    env::verify(nomos_cl_risc0_proofs::PTX_ID, &serde::to_vec(&ptx).unwrap()).unwrap();

    let ptx_cm_root = ptx.cm_root;
    let ptx = &ptx.ptx;

    // TODO: accept inputs from multiple zones
    let check_inputs = ptx.inputs.iter().all(|input| input.zone_id == zone_id);

    if check_inputs {
        assert_eq!(ptx_cm_root, cm_root);
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
    }

    for output in &ptx.outputs {
        if output.zone_id == zone_id {
            ledger.commitments.push(output.note_comm);
            // if this output was not originating from this zone, it is a cross zone transaction
            if !check_inputs {
                cross_in.push(*output);
            }
        } else {
            // if this output is not going to this zone but originated from this zone, it is a cross zone transaction
            if check_inputs {
                cross_out.push(*output);
            }
        }
    }

    (ledger, cross_in, cross_out)
}
