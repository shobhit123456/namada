//! Anoma client CLI.

use std::fs::File;
use std::io::Write;

use anoma::cli::{args, cmds};
use anoma::client::tx;
use anoma::proto::services::rpc_service_client::RpcServiceClient;
use anoma::proto::{services, RpcMessage};
use anoma::{cli, wallet};
use anoma_shared::types::intent::{
    DecimalWrapper, Exchange, FungibleTokenIntent,
};
use anoma_shared::types::key::ed25519::Signed;
use anoma_shared::types::token::Amount;
use borsh::BorshSerialize;
use color_eyre::eyre::Result;

pub async fn main() -> Result<()> {
    let (cmd, _global_args) = cli::anoma_client_cli();
    match cmd {
        cmds::AnomaClient::TxCustom(cmds::TxCustom(args)) => {
            tx::submit_custom(args).await;
        }
        cmds::AnomaClient::TxTransfer(cmds::TxTransfer(args)) => {
            tx::submit_transfer(args).await;
        }
        cmds::AnomaClient::TxUpdateVp(cmds::TxUpdateVp(args)) => {
            tx::submit_update_vp(args).await;
        }
        cmds::AnomaClient::Intent(cmds::Intent(args)) => {
            gossip_intent(args).await;
        }
        cmds::AnomaClient::CraftIntent(cmds::CraftIntent(args)) => {
            craft_intent(args);
        }
        cmds::AnomaClient::SubscribeTopic(cmds::SubscribeTopic(args)) => {
            subscribe_topic(args).await;
        }
    }
    Ok(())
}

async fn gossip_intent(
    args::Intent {
        node_addr,
        data_path,
        topic,
    }: args::Intent,
) {
    let mut client = RpcServiceClient::connect(node_addr).await.unwrap();
    let data = std::fs::read(data_path).expect("data file IO error");
    let intent = anoma_shared::proto::Intent::new(data);
    let message: services::RpcMessage =
        RpcMessage::new_intent(intent, topic).into();
    let response = client
        .send_message(message)
        .await
        .expect("failed to send message and/or receive rpc response");
    println!("{:#?}", response);
}

async fn subscribe_topic(
    args::SubscribeTopic { node_addr, topic }: args::SubscribeTopic,
) {
    let mut client = RpcServiceClient::connect(node_addr).await.unwrap();
    let message: services::RpcMessage = RpcMessage::new_topic(topic).into();
    let response = client
        .send_message(message)
        .await
        .expect("failed to send message and/or receive rpc response");
    println!("{:#?}", response);
}

fn craft_intent(
    args::CraftIntent {
        addr,
        token_sell,
        max_sell,
        min_rate,
        token_buy,
        min_buy,
        file_path,
    }: args::CraftIntent,
) {
    let source_keypair = wallet::key_of(&addr.encode());

    let exchange = Exchange {
        addr,
        token_sell,
        token_buy,
        min_buy,
        rate_min: min_rate,
        max_sell,
    };
    let signed_exchange: Signed<Exchange> =
        Signed::new(&source_keypair, exchange);
    let signed_ft: Signed<FungibleTokenIntent> = Signed::new(
        &source_keypair,
        FungibleTokenIntent {
            exchange: vec![signed_exchange].into_iter().collect(),
        },
    );
    let data_bytes = signed_ft.try_to_vec().unwrap();

    let mut file = File::create(file_path).unwrap();
    file.write_all(&data_bytes).unwrap();
}
