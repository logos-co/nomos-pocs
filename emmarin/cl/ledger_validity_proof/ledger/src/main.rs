use cl::{
    cl::{Bundle, Output},
    zone_layer::{ledger::LedgerWitness, notes::ZoneId},
};
use ledger_proof_statements::{
    balance::BalancePublic,
    constraint::ConstraintPublic,
    ledger::{CrossZoneBundle, LedgerProofPrivate, LedgerProofPublic},
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
    let mut cross_bundles = vec![];
    let mut outputs = vec![];

    let roots = ledger
        .commitments
        .roots
        .iter()
        .map(|r| r.root)
        .collect::<Vec<_>>();

    for bundle in bundles {
        let balance_public = BalancePublic {
            balances: bundle.iter().map(|ptx| ptx.ptx.balance).collect::<Vec<_>>(),
        };
        // verify bundle is balanced
        env::verify(
            nomos_cl_risc0_proofs::BALANCE_ID,
            &serde::to_vec(&balance_public).unwrap(),
        )
        .unwrap();

        for ptx in &bundle {
            let (new_ledger, ptx_outputs) = process_ptx(ledger, ptx, id, &roots);
            ledger = new_ledger;
            outputs.extend(ptx_outputs);
        }

        let bundle = Bundle {
            partials: bundle.into_iter().map(|ptx| ptx.ptx).collect(),
        };
        let zones = bundle.zones();
        if zones.len() > 1 {
            cross_bundles.push(CrossZoneBundle {
                id: bundle.id(),
                zones: zones.into_iter().collect(),
            });
        }
    }

    env::commit(&LedgerProofPublic {
        old_ledger,
        ledger: ledger.commit(),
        id,
        cross_bundles,
        outputs,
    });
}

fn process_ptx(
    mut ledger: LedgerWitness,
    ptx: &PtxPublic,
    zone_id: ZoneId,
    roots: &[[u8; 32]],
) -> (LedgerWitness, Vec<Output>) {
    // always verify the ptx to ensure outputs were derived with the correct zone id
    env::verify(nomos_cl_risc0_proofs::PTX_ID, &serde::to_vec(&ptx).unwrap()).unwrap();

    let cm_roots = &ptx.cm_roots;
    let ptx = &ptx.ptx;

    let mut outputs = vec![];

    for (input, input_cm_root) in ptx.inputs.iter().zip(cm_roots) {
        if input.zone_id == zone_id {
            assert!(roots.contains(input_cm_root));
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
            ledger.commitments.push(&output.note_comm.0);
            outputs.push(*output);
        }
    }

    (ledger, outputs)
}
