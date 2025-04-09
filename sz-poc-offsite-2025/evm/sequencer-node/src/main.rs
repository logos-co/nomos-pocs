use reth::cli::Cli;
use reth_node_ethereum::node::{EthereumAddOns, EthereumNode};

use evm_sequencer_mempool::EvmSequencerMempoolBuilder;

fn main() {
    Cli::parse_args()
        .run(|builder, _| async move {
            // launch the node
            let handle = Box::pin(
                builder
                    // use the default ethereum node types
                    .with_types::<EthereumNode>()
                    // use default ethereum components but use our custom pool
                    .with_components(
                        EthereumNode::components().pool(EvmSequencerMempoolBuilder::default()),
                    )
                    .with_add_ons(EthereumAddOns::default())
                    .launch(),
            )
            .await?;

            handle.wait_for_node_exit().await
        })
        .unwrap();
}
