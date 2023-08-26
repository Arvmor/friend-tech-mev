use std::{sync::Arc, ops::{Add, Mul, Div}, error::Error};

use ethers::{
    types::{Eip1559TransactionRequest, BlockNumber, U64, U256},
    providers::{Http, Provider, Middleware}, signers::Wallet, prelude::{*, k256::ecdsa::SigningKey}
};
use eyre::Result;
use tokio::sync::mpsc::Sender;

use crate::{api::{get_user, get_user_followers}, builder::build_buy_transaction};

pub async fn send_trx(
    client: SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>>,
    mut transaction: Eip1559TransactionRequest,
    block_number: U64,
    base_fee: U256
) -> Result<()> {
    transaction = transaction.max_fee_per_gas(base_fee).max_priority_fee_per_gas(base_fee);

    match client.send_transaction(transaction, Some(BlockNumber::Number(block_number.add(1)).into())).await {
        Ok(info) => {
            match info.await {
                Ok(info2) => println!("Success: {:#?}", info2.unwrap().transaction_hash),
                Err(error) => eprintln!("failed {:#?}", error)
            }
        },
        Err(error) => eprintln!("failed {:#?}", error)
    }
    Ok(())
}

pub async fn runner(
    watchlist: Vec<H160>,
    results: (H160, H160, bool, U256, U256, U256, U256, U256),
    client: SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>>,
    nonce: U256,
    block_number: U64,
    base_fee: U256
) -> Result<(), Box<dyn Error + Send>> {

    if watchlist.contains(&results.1) {
        println!("SENDING !");
        tokio::spawn(
            send_trx(client.clone(), build_buy_transaction(results.1, U256::from(1), results.7, nonce), block_number, base_fee.mul(700u16).div(10))
        );
    }

    Ok(())
}

pub async fn add_to_watchlist(
    address: H160,
    sender: Sender<H160>
) -> Result<(), Box<dyn Error + Send>> {

    println!("Victim {:#?}", address);
    let username = get_user(address, 100).await.unwrap();
    println!("ID: @{:#?}", username);
    let follower_count = get_user_followers(&username).await.unwrap();
    println!("Follower Count: {:#?}", follower_count);

    if follower_count > 1000 {
        println!("Added ! {:#?}", address);
        sender.send(address).await.expect("Failed to add address");
    }

    Ok(())
}