use clap::Parser;
use futures_util::StreamExt;
use reth::{
    builder::NodeHandle, chainspec::EthereumChainSpecParser, cli::Cli,
    transaction_pool::TransactionPool,
};
use reth_cli_commands::node::NoArgs;
use reth_node_ethereum::node::EthereumNode;

fn main() {
    Cli::<EthereumChainSpecParser, NoArgs>::parse()
        .run(|builder, _| async move {
            // launch the node
            let NodeHandle {
                node,
                node_exit_future,
            } = builder.node(EthereumNode::default()).launch().await?;

            // create a new subscription to pending transactions
            let mut pending_transactions = node.pool.new_pending_pool_transactions_listener();

            // Spawn an async block to listen for validated transactions.
            node.task_executor.spawn(Box::pin(async move {
                // Waiting for new transactions
                while let Some(event) = pending_transactions.next().await {
                    let tx = event.transaction;
                    println!("Transaction received: {:?}", tx.transaction);
                }
            }));

            node_exit_future.await
        })
        .unwrap();
}
