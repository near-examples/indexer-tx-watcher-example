use std::str::FromStr;

use std::collections::{HashMap, HashSet};

use clap::Clap;
use tokio::sync::mpsc;
use tracing::info;

use configs::{init_logging, Opts, SubCommand};

mod configs;

/// Assuming we want to watch for transactions where a receiver account id is one of the provided in a list
/// We pass the list of account ids (or contracts it is the same) via argument ``--accounts``
/// We want to catch all *successfull* transactions sent to one of the accounts from the list.
/// In the demo we'll just look for them and log them but it might and probably should be extended based on your needs.

fn main() {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    openssl_probe::init_ssl_cert_env_vars();
    init_logging();

    let opts: Opts = Opts::parse();

    let home_dir = opts.home_dir.unwrap_or_else(near_indexer::get_default_home);

    match opts.subcmd {
        SubCommand::Run(args) => {
            // Create the Vec of AccountId from the provided ``--accounts`` to pass it to `listen_blocks`
            let watching_list = args
                .accounts
                .split(',')
                .map(|elem| {
                    near_indexer::near_primitives::types::AccountId::from_str(elem)
                        .expect("AccountId is invalid")
                })
                .collect();

            // Inform about indexer is being started and what accounts we're watching for
            eprintln!(
                "Starting indexer transaction watcher for accounts: \n {:#?}",
                &args.accounts
            );

            // Instantiate IndexerConfig with hardcoded parameters
            let indexer_config = near_indexer::IndexerConfig {
                home_dir,
                sync_mode: near_indexer::SyncModeEnum::FromInterruption,
                await_for_node_synced: near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync,
            };

            // Boilerplate code to start the indexer itself
            let sys = actix::System::new();
            sys.block_on(async move {
                eprintln!("Actix");
                let indexer = near_indexer::Indexer::new(indexer_config);
                let stream = indexer.streamer();
                actix::spawn(listen_blocks(stream, watching_list));
            });
            sys.run().unwrap();
        }
        SubCommand::Init(config) => near_indexer::indexer_init_configs(&home_dir, config.into()),
    }
}

/// The main listener function the will be reading the stream of blocks `StreamerMessage`
/// and perform necessary checks
async fn listen_blocks(
    mut stream: mpsc::Receiver<near_indexer::StreamerMessage>,
    watching_list: Vec<near_indexer::near_primitives::types::AccountId>,
) {
    eprintln!("listen_blocks");
    // This will be a map of correspondence between transactions and receipts
    let mut tx_receipt_ids = HashMap::<String, String>::new();
    // This will be a list of receipt ids we're following
    let mut wanted_receipt_ids = HashSet::<String>::new();

    // Boilerplate code to listen the stream
    while let Some(streamer_message) = stream.recv().await {
        eprintln!("Block height: {}", streamer_message.block.header.height);
        for shard in streamer_message.shards {
            let chunk = if let Some(chunk) = shard.chunk {
                chunk
            } else {
                continue;
            };

            for transaction in chunk.transactions {
                // Check if transaction receiver id is one of the list we are interested in
                if is_tx_receiver_watched(&transaction, &watching_list) {
                    // extract receipt_id transaction was converted into
                    let converted_into_receipt_id = transaction
                        .outcome
                        .execution_outcome
                        .outcome
                        .receipt_ids
                        .first()
                        .expect("`receipt_ids` must contain one Receipt Id")
                        .to_string();
                    // add `converted_into_receipt_id` to the list of receipt ids we are interested in
                    wanted_receipt_ids.insert(converted_into_receipt_id.clone());
                    // add key value pair of transaction hash and in which receipt id it was converted for further lookup
                    tx_receipt_ids.insert(
                        converted_into_receipt_id,
                        transaction.transaction.hash.to_string(),
                    );
                }
            }

            for execution_outcome in shard.receipt_execution_outcomes {
                if let Some(receipt_id) =
                    wanted_receipt_ids.take(&execution_outcome.receipt.receipt_id.to_string())
                {
                    // log the tx because we've found it
                    info!(
                        target: "indexer_example",
                        "Transaction hash {:?} related to {} executed with status {:?}",
                        tx_receipt_ids.get(receipt_id.as_str()),
                        &execution_outcome.receipt.receiver_id,
                        execution_outcome.execution_outcome.outcome.status
                    );
                    if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                        signer_id,
                        ..
                    } = &execution_outcome.receipt.receipt
                    {
                        eprintln!("{}", signer_id);
                    }

                    if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                        actions,
                        ..
                    } = execution_outcome.receipt.receipt
                    {
                        for action in actions.iter() {
                            if let near_indexer::near_primitives::views::ActionView::FunctionCall {
                                args,
                                ..
                            } = action
                            {
                                if let Ok(decoded_args) = base64::decode(args) {
                                    if let Ok(args_json) = serde_json::from_slice::<serde_json::Value>(&decoded_args) {
                                        eprintln!("{:#?}", args_json);
                                    }
                                }
                            }
                        }
                    }
                    // remove tx from hashmap
                    tx_receipt_ids.remove(receipt_id.as_str());
                }
            }
        }
    }
}

fn is_tx_receiver_watched(
    tx: &near_indexer::IndexerTransactionWithOutcome,
    watching_list: &[near_indexer::near_primitives::types::AccountId],
) -> bool {
    watching_list.contains(&tx.transaction.receiver_id)
}
