use cl::{
    cl::{Bundle, Output},
    zone_layer::{ledger::LedgerWitness, notes::ZoneId},
};
use ledger_proof_statements::{
    balance::BalancePublic,
    ledger::{CrossZoneBundle, LedgerProofPrivate, LedgerProofPublic, LedgerPtxWitness},
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

    for bundle in bundles {
        let balance_public = BalancePublic {
            balances: bundle.partials.iter().map(|bundle_ptx| bundle_ptx.ptx.ptx.balance).collect::<Vec<_>>(),
        };

        // verify bundle is balanced
        env::verify(
            nomos_cl_risc0_proofs::BALANCE_ID,
            &serde::to_vec(&balance_public).unwrap(),
        )
        .unwrap();

        for ptx in &bundle.partials {
            let ptx_outputs = process_ptx(&mut ledger, ptx, id);
            outputs.extend(ptx_outputs);
        }

        let bundle = Bundle {
            partials: bundle.partials.into_iter().map(|ptx_witness| ptx_witness.ptx.ptx).collect(),
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
    ledger: &mut LedgerWitness,
    ptx_witness: &LedgerPtxWitness,
    zone_id: ZoneId,
) -> Vec<Output> {
    let ptx = &ptx_witness.ptx;
    let nf_proofs = &ptx_witness.nf_proofs;

    // always verify the ptx to ensure outputs were derived with the correct zone id
    env::verify(nomos_cl_risc0_proofs::PTX_ID, &serde::to_vec(&ptx).unwrap()).unwrap();

    
    assert_eq!(ptx.ptx.inputs.len(), nf_proofs.len());
    assert_eq!(ptx.ptx.inputs.len(), ptx.cm_mmr.len());

    for ((input, nf_proof), cm_mmr) in ptx.ptx.inputs.iter().zip(nf_proofs).zip(ptx.cm_mmr.iter()) {
        if input.zone_id != zone_id {
            continue;
        }
        
        assert_eq!(cm_mmr, &ledger.commitments); // we force commitment proofs w.r.t. latest MMR

        ledger.assert_nf_update(input.nullifier, nf_proof);
    }

    let mut outputs = vec![];
    for output in &ptx.ptx.outputs {
        if output.zone_id != zone_id {
            continue;
        }

        ledger.commitments.push(&output.note_comm.0);
        outputs.push(*output);
    }

    outputs
}
