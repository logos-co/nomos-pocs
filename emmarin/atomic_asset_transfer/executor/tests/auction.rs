use std::collections::BTreeMap;

use cl::{BalanceWitness, BundleWitness, Nonce, NoteWitness};
use common::{
    height_to_segment, new_account, segment_to_height, Bid, BoundTx, Deposit, SignedBoundTx, Tx,
    Withdraw, BID_END_OFFSET, BID_START_OFFSET, REVEAL_END_OFFSET,
};
use executor::ZoneNotes;
use goas_proof_statements::user_note::{UserAtomicTransfer, UserIntent};
use rand::seq::SliceRandom;

#[test]
#[should_panic]
fn test_auction_bid_before_bidding_range() {
    let mut rng = rand::thread_rng();

    let zone_start = ZoneNotes::new_with_balances("ZONE_A", BTreeMap::from_iter([]), &mut rng);

    let target_block = 1542;
    let target_segment = height_to_segment(target_block);

    assert_eq!(target_segment, 154);

    let mut rng = rand::thread_rng();

    let mut bob = new_account(&mut rng);
    let bob_vk = bob.verifying_key().to_bytes();

    let bob_bid = Bid {
        segment: target_segment,
        exec_vk: bob_vk,
        fee: 500,
    };

    let tx_blind_bid = Tx::TicketBlindBid(bob_bid.commit());

    let current_height = 100;

    let (zone_end, bid_inclusion) = zone_start.clone().run(current_height, tx_blind_bid);
}

#[test]
fn test_auction_happy_path_single_exec() {
    let mut rng = rand::thread_rng();

    let zone_init_state = ZoneNotes::new_with_balances("ZONE_A", BTreeMap::from_iter([]), &mut rng);

    let target_block = 1542;
    let target_segment = height_to_segment(target_block);

    assert_eq!(target_segment, 154);

    let mut rng = rand::thread_rng();

    let mut bob = new_account(&mut rng);
    let bob_vk = bob.verifying_key().to_bytes();

    let bob_bid = Bid {
        segment: target_segment,
        exec_vk: bob_vk,
        fee: 500,
    };

    let tx_blind_bid = Tx::TicketBlindBid(bob_bid.commit());

    let bid_height = segment_to_height(target_segment) - BID_START_OFFSET;

    let (zone_blind_bid_state, bid_inclusion) =
        zone_init_state.clone().run(bid_height, tx_blind_bid);

    assert!(zone_blind_bid_state
        .state
        .ticket_auction
        .blind_bids
        .get(&target_segment)
        .unwrap()
        .contains(&bob_bid.commit()));

    let tx_reveal_bid = Tx::TicketRevealBid(bob_bid);

    let reveal_height = segment_to_height(target_segment) - REVEAL_END_OFFSET - 1;

    let (zone_reveal_bid_state, reveal_inclusion) = zone_blind_bid_state
        .clone()
        .run(reveal_height, tx_reveal_bid);

    assert!(zone_reveal_bid_state
        .state
        .ticket_auction
        .blind_bids
        .get(&target_segment)
        .unwrap()
        .contains(&bob_bid.commit()));

    let dutch_bid = zone_reveal_bid_state
        .state
        .ticket_auction
        .revealed_bids
        .get(&target_segment)
        .unwrap();

    assert!(dutch_bid.scd_best == bob_bid);
    assert!(dutch_bid.best == bob_bid);
}

#[test]
fn test_auction_happy_path_multi_exec() {
    let mut rng = rand::thread_rng();

    let zone_init_state = ZoneNotes::new_with_balances("ZONE_A", BTreeMap::from_iter([]), &mut rng);

    let target_block = 1542;
    let target_segment = height_to_segment(target_block);

    assert_eq!(target_segment, 154);

    let mut rng = rand::thread_rng();

    let mut nums = Vec::from_iter(0..100);
    nums.shuffle(&mut rng);

    let mut bids = Vec::from_iter(nums.iter().map(|n| {
        let mut executor_sk = new_account(&mut rng);
        let executor_vk = executor_sk.verifying_key().to_bytes();

        Bid {
            segment: target_segment,
            exec_vk: executor_vk,
            fee: *n,
        }
    }));

    let mut zone_curr_state = zone_init_state;
    for (i, executor_bid) in bids.iter().enumerate() {
        let tx_blind_bid = Tx::TicketBlindBid(executor_bid.commit());

        let bid_height = segment_to_height(target_segment) - BID_START_OFFSET
            + (i as u64 % (BID_START_OFFSET - BID_END_OFFSET));

        (zone_curr_state, _) = zone_curr_state.clone().run(bid_height, tx_blind_bid);
    }

    for bid in bids.iter() {
        assert!(zone_curr_state
            .state
            .ticket_auction
            .blind_bids
            .get(&target_segment)
            .unwrap()
            .contains(&bid.commit()));
    }

    bids.shuffle(&mut rng);

    for (i, bid) in bids.iter().enumerate() {
        let tx_reveal_bid = Tx::TicketRevealBid(bid.clone());

        let reveal_height = segment_to_height(target_segment) - BID_END_OFFSET
            + (i as u64 % (BID_END_OFFSET - REVEAL_END_OFFSET));

        (zone_curr_state, _) = zone_curr_state.clone().run(reveal_height, tx_reveal_bid);
    }

    for bid in bids.iter() {
        assert!(zone_curr_state
            .state
            .ticket_auction
            .blind_bids
            .get(&target_segment)
            .unwrap()
            .contains(&bid.commit()));
    }

    let dutch_bid = zone_curr_state
        .state
        .ticket_auction
        .revealed_bids
        .get(&target_segment)
        .unwrap();

    bids.sort_by_key(|b| b.fee);

    assert_eq!(dutch_bid.scd_best, bids[bids.len() - 2]);
    assert_eq!(dutch_bid.best, bids[bids.len() - 1]);
}
