//! Remote WebSocket Account Ingestion Adapter
//! 
//! Connects to standard remote RPC WebSockets (Helius, Alchemy, QuickNode)
//! and feeds account updates directly into the bot's MessagesV2 pipeline.
//! Eliminates the requirement for a local Solana validator and ZeroMQ IPC socket.

use futures::StreamExt;
use log::{error, info, warn};
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use utils::deserialize::{Message, MessagesV2};

/// Subscribes to a set of accounts via remote RPC WebSocket and forwards updates to `tx`.
pub async fn stream_accounts_ws(
    ws_url: String,
    accounts: Vec<Pubkey>,
    tx: crossbeam_channel::Sender<MessagesV2>,
) {
    if accounts.is_empty() {
        warn!("WebSocket streamer: No accounts specified to monitor.");
        return;
    }

    info!(
        "Connecting WebSocket account stream to {} for {} accounts...",
        ws_url,
        accounts.len()
    );

    let config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        commitment: Some(CommitmentConfig::processed()),
        data_slice: None,
        min_context_slot: None,
    };

    for pubkey in accounts {
        let ws_url_clone = ws_url.clone();
        let tx_clone = tx.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            loop {
                match PubsubClient::new(&ws_url_clone).await {
                    Ok(client) => {
                        info!("Subscribed to account {} via WebSocket", pubkey);
                        match client.account_subscribe(&pubkey, Some(config_clone.clone())).await {
                            Ok((mut stream, _unsub)) => {
                                while let Some(response) = stream.next().await {
                                    let slot = response.context.slot;
                                    let ui_account = response.value;

                                    let data = match ui_account.data.decode() {
                                        Some(bytes) => bytes,
                                        None => {
                                            warn!("Failed to decode base64 account data for {}", pubkey);
                                            continue;
                                        }
                                    };

                                    let owner = match Pubkey::from_str(&ui_account.owner) {
                                        Ok(pk) => pk,
                                        Err(_) => {
                                            warn!("Invalid owner pubkey for {}", pubkey);
                                            continue;
                                        }
                                    };

                                    let msg = Message {
                                        pubkey,
                                        owner,
                                        data,
                                    };

                                    let msgs_v2 = MessagesV2 {
                                        message: vec![msg],
                                        slot,
                                    };

                                    if let Err(e) = tx_clone.send(msgs_v2) {
                                        error!("Failed to forward WebSocket update to channel: {:?}", e);
                                        return;
                                    }
                                }
                                warn!("WebSocket stream ended for {}, reconnecting...", pubkey);
                            }
                            Err(e) => {
                                warn!("Failed to subscribe to account {}: {:?}. Retrying...", pubkey, e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("WebSocket connection to {} failed: {:?}. Retrying in 5s...", ws_url_clone, e);
                    }
                }
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}

/// Returns a default list of highly active Solana DEX pool accounts to monitor via WebSocket.
pub fn get_default_monitored_pools() -> Vec<Pubkey> {
    let pool_strs = [
        "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2", // Raydium AMM SOL/USDC
        "7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX", // Raydium AMM SOL/USDT
        "HJPjoWUrhoZzkNfRpHuieeFKAvVQDDUsPmeeNTpaafP", // Orca Whirlpool SOL/USDC
        "4BpEtYPZcEGU7KxN9jPHkEWeA4TdQ9n8Wb8p6j1oUSiY", // Orca Whirlpool SOL/USDT
        "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA", // Raydium AMM RAY/SOL
        "ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq", // Meteora DLMM SOL/USDC
    ];
    pool_strs
        .iter()
        .filter_map(|s| Pubkey::from_str(s).ok())
        .collect()
}
