use evm_aggregator::Aggregator;
use futures::TryStreamExt as _;
use reth::{
    api::{FullNodeTypes, NodePrimitives, NodeTypes},
    cli::Cli,
};
use reth_ethereum::{
    exex::{ExExContext, ExExEvent, ExExNotification},
    node::{EthereumNode, api::FullNodeComponents},
};
use reth_tracing::tracing::info;

async fn aggregate_block_txs<Node: FullNodeComponents>(
    mut ctx: ExExContext<Node>,
    mut aggregator: Aggregator,
) -> eyre::Result<()>
where
    <<Node as FullNodeTypes>::Types as NodeTypes>::Primitives:
        NodePrimitives<Block = reth_ethereum::Block>,
{
    while let Some(notification) = ctx.notifications.try_next().await? {
        let ExExNotification::ChainCommitted { new } = &notification else {
            continue;
        };
        info!(committed_chain = ?new.range(), "Received commit");
        aggregator.process_blocks(
            new.inner()
                .0
                .clone()
                .into_blocks()
                .map(reth_ethereum::primitives::RecoveredBlock::into_block),
        );

        if let Some(committed_chain) = notification.committed_chain() {
            ctx.events
                .send(ExExEvent::FinishedHeight(committed_chain.tip().num_hash()))
                .unwrap();
        }
    }

    Ok(())
}

fn main() -> eyre::Result<()> {
    Cli::try_parse_args_from([
        "reth",
        "node",
        "--datadir=/tmp/reth-dev/",
        "--dev",
        "--dev.block-time=2s",
        "--http.addr=0.0.0.0",
    ])
    .unwrap()
    .run(|builder, _| {
        Box::pin(async move {
            let aggregator = Aggregator::default();
            let handle = Box::pin(
                builder
                    .node(EthereumNode::default())
                    .install_exex("aggregate-block-txs", async move |ctx| {
                        Ok(aggregate_block_txs(ctx, aggregator))
                    })
                    .launch(),
            )
            .await?;

            handle.wait_for_node_exit().await
        })
    })
}
