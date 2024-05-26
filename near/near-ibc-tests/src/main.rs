mod msgs;
mod utils;

use std::{thread::sleep, time::Duration};

use ibc_vm_rs::{states::connection_handshake, DEFAULT_IBC_VERSION, DEFAULT_MERKLE_PREFIX};
use near_primitives_core::hash::CryptoHash;
use near_workspaces::{
    network::Sandbox,
    sandbox,
    types::{Gas, KeyType, NearToken, SecretKey},
    Account, AccountId, Contract, Worker,
};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    ibc::core::{
        channel::{self, channel::Channel},
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection,
    },
    near::types::HeaderUpdate,
    validated::ValidateT,
};
use utils::convert_block_producers;

use crate::{
    msgs::{
        ChannelOpenAck, ChannelOpenInit, ClientState, ConnectionOpenAck, ConnectionOpenInit,
        ConnectionOpenTry, ConsensusState, CreateClient, GetAccountId, GetCommitment,
        RegisterClient, UpdateClient,
    },
    utils::{
        chunk_proof, convert_block_header_inner, convert_light_client_block_view, state_proof,
    },
};

const IBC_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/near_ibc.wasm";
const LC_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/near_light_client.wasm";
const IBC_APP_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/dummy_ibc_app.wasm";

mod alice {
    pub const CLIENT_TYPE: &str = "near-alice";
}
mod bob {
    pub const CLIENT_TYPE: &str = "near-bob";
}

const INITIAL_HEIGHT: Height = Height {
    revision_number: 0,
    revision_height: 100,
};

struct NearContract {
    account_id: AccountId,
    secret_key: SecretKey,
    contract: Contract,
}

pub async fn deploy_contract(
    sandbox: &Worker<Sandbox>,
    account_id: &str,
    wasm_path: &'static str,
) -> Contract {
    let wasm_blob = std::fs::read(wasm_path).unwrap();
    let account_id = account_id.to_string().try_into().unwrap();
    let secret_key = SecretKey::from_seed(KeyType::ED25519, "testificate");
    sandbox
        .create_tla_and_deploy(account_id, secret_key, &wasm_blob)
        .await
        .unwrap()
        .unwrap()
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let sandbox = sandbox().await.unwrap();

    let ibc_contract = deploy_contract(&sandbox, "ibc.test.near", IBC_WASM_FILEPATH).await;
    let alice_lc = deploy_contract(&sandbox, "light-client.test.near", LC_WASM_FILEPATH).await;
    let bob_lc = deploy_contract(
        &sandbox,
        "counterparty-light-client.test.near",
        LC_WASM_FILEPATH,
    )
    .await;
    let _ibc_app_contract =
        deploy_contract(&sandbox, "ibc-app.test.near", IBC_APP_WASM_FILEPATH).await;

    // create accounts
    let owner = sandbox.root_account().unwrap();
    let user = owner
        .create_subaccount("user")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await
        .unwrap()
        .into_result()
        .unwrap();

    println!("calling register");

    register_client(
        &user,
        &ibc_contract,
        &alice_lc,
        alice::CLIENT_TYPE.to_string(),
    )
    .await;
    register_client(&user, &ibc_contract, &bob_lc, bob::CLIENT_TYPE.to_string()).await;

    create_client(
        &sandbox,
        &user,
        &ibc_contract,
        alice::CLIENT_TYPE.to_string(),
    )
    .await;
    create_client(&sandbox, &user, &ibc_contract, bob::CLIENT_TYPE.to_string()).await;

    connection_open(&sandbox, &user, &ibc_contract, &alice_lc, &bob_lc).await;
}

async fn connection_open(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    alice_lc: &Contract,
    bob_lc: &Contract,
) {
    let alice_client_id = format!("{}-1", alice::CLIENT_TYPE);
    let bob_client_id = format!("{}-2", bob::CLIENT_TYPE);
    let open_init = ConnectionOpenInit {
        client_id: alice_client_id.clone(),
        counterparty: connection_handshake::Counterparty {
            client_id: bob_client_id.clone().validate().unwrap(),
            connection_id: "".into(),
            prefix: MerklePrefix {
                key_prefix: b"ibc".into(),
            },
        },
        version: DEFAULT_IBC_VERSION[0].clone(),
        delay_period: 0,
    };

    println!("calling connection open init on alice");
    let res = user
        .call(ibc_contract.id(), "connection_open_init")
        .args_json(open_init)
        .transact()
        .await
        .unwrap();
    println!("connection open init res: {:?}", res);

    let latest_height: Height = serde_json::from_slice(
        &user
            .view(bob_lc.id(), "latest_height")
            .await
            .unwrap()
            .result,
    )
    .unwrap();

    let current_block = sandbox.view_block().await.unwrap();
    sleep(Duration::from_secs(2));

    let light_client_block = sandbox
        .next_light_client_block(current_block.hash())
        .await
        .unwrap();

    let current_height = light_client_block.inner_lite.height;

    let (prev_state_root, prev_state_root_proof) = chunk_proof(sandbox, current_height).await;

    let update_bob = UpdateClient {
        client_id: bob_client_id.clone(),
        client_msg: borsh::to_vec(&HeaderUpdate {
            new_state: convert_light_client_block_view(light_client_block),
            trusted_height: latest_height.revision_height,
            prev_state_root_proof,
            prev_state_root,
        })
        .unwrap(),
    };

    let res = user
        .call(ibc_contract.id(), "update_client")
        .args_json(update_bob)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap()
        .unwrap();

    println!("Update result: {res:?}");

    let connection_end_proof = state_proof(
        sandbox,
        ibc_contract.id(),
        current_height - 1,
        "connections/connection-1",
    )
    .await;

    let open_try = ConnectionOpenTry {
        client_id: bob_client_id.clone(),
        counterparty: connection_handshake::Counterparty {
            client_id: alice_client_id.clone().validate().unwrap(),
            connection_id: "connection-1".into(),
            prefix: MerklePrefix {
                key_prefix: b"ibc".into(),
            },
        },
        counterparty_versions: DEFAULT_IBC_VERSION.clone(),
        connection_end_proof,
        proof_height: Height {
            revision_number: 0,
            revision_height: current_height - 1,
        },
        delay_period: 0,
    };

    println!("calling connection open try on bob");
    let res = user
        .call(ibc_contract.id(), "connection_open_try")
        .args_json(open_try)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap();
    println!("connection open try res: {:?}", res);

    // sandbox.fast_forward(1).await.unwrap();
    // let current_block = sandbox.view_block().await.unwrap();
    // let mut proof_key = b"commitments".to_vec();
    // proof_key.extend(borsh::to_vec("connections/connection-2").unwrap());

    // let state = sandbox
    //     .view_state(ibc_contract.id())
    //     .prefix(&proof_key)
    //     .block_height(current_block.height() - 1)
    //     .await
    //     .unwrap();
    // println!("got the proof data: {:?}", state.data);
    // let state_proof: Vec<Vec<u8>> = state
    //     .proof
    //     .clone()
    //     .into_iter()
    //     .map(|item| item.to_vec())
    //     .collect();

    // let update_alice = UpdateClient {
    //     client_id: alice_client_id.clone(),
    //     client_msg: borsh::to_vec(&(
    //         current_block.height() - 1,
    //         ConsensusState {
    //             state_root: CryptoHash(current_block.chunks()[0].prev_state_root.0.clone()),
    //         },
    //     ))
    //     .unwrap(),
    // };

    // let res = user
    //     .call(ibc_contract.id(), "update_client")
    //     .args_json(update_alice)
    //     .gas(Gas::from_gas(300000000000000))
    //     .transact()
    //     .await
    //     .unwrap()
    //     .unwrap();

    // println!("Update result: {res:?}");

    // let open_ack = ConnectionOpenAck {
    //     connection_id: "connection-1".into(),
    //     version: DEFAULT_IBC_VERSION[0].clone(),
    //     counterparty_connection_id: "connection-2".into(),
    //     connection_end_proof: serde_json::to_vec(&state_proof).unwrap(),
    //     proof_height: Height {
    //         revision_number: 0,
    //         revision_height: current_block.height() - 1,
    //     },
    // };

    // println!("calling connection open ack on alice");
    // let res = user
    //     .call(ibc_contract.id(), "connection_open_ack")
    //     .args_json(open_ack)
    //     .gas(Gas::from_gas(300000000000000))
    //     .transact()
    //     .await
    //     .unwrap();
    // println!("connection open ack res: {:?}", res);

    // sandbox.fast_forward(1).await.unwrap();
    // let current_block = sandbox.view_block().await.unwrap();
    // let mut proof_key = b"commitments".to_vec();
    // proof_key.extend(borsh::to_vec("connections/connection-1").unwrap());

    // let state = sandbox
    //     .view_state(ibc_contract.id())
    //     .prefix(&proof_key)
    //     .block_height(current_block.height() - 1)
    //     .await
    //     .unwrap();
    // println!("got the proof data: {:?}", state.data);
    // let state_proof: Vec<Vec<u8>> = state
    //     .proof
    //     .clone()
    //     .into_iter()
    //     .map(|item| item.to_vec())
    //     .collect();

    // let update_bob = UpdateClient {
    //     client_id: bob_client_id.clone(),
    //     client_msg: borsh::to_vec(&(
    //         current_block.height() - 1,
    //         ConsensusState {
    //             state_root: CryptoHash(current_block.chunks()[0].prev_state_root.0.clone()),
    //         },
    //     ))
    //     .unwrap(),
    // };

    // let res = user
    //     .call(ibc_contract.id(), "update_client")
    //     .args_json(update_bob)
    //     .gas(Gas::from_gas(300000000000000))
    //     .transact()
    //     .await
    //     .unwrap()
    //     .unwrap();

    // println!("Update result: {res:?}");

    // let open_confirm = ConnectionOpenConfirm {
    //     connection_id: "connection-2".to_string(),
    //     connection_end_proof: serde_json::to_vec(&state_proof).unwrap(),
    //     proof_height: Height {
    //         revision_number: 0,
    //         revision_height: current_block.height() - 1,
    //     },
    // };

    // println!("calling connection open confirm on bob");
    // let res = user
    //     .call(ibc_contract.id(), "connection_open_confirm")
    //     .args_json(open_confirm)
    //     .gas(Gas::from_gas(300000000000000))
    //     .transact()
    //     .await
    //     .unwrap();
    // println!("connection open confirm res: {:?}", res);

    // println!("[ + ] Connection opened between {alice_client_id} and {bob_client_id}");
}

/// Expectations:
/// 1. Light client's account id should be saved under the key `client_type`
async fn register_client(user: &Account, contract: &Contract, lc: &Contract, client_type: String) {
    let register = RegisterClient {
        client_type: client_type.clone(),
        account: lc.id().to_string(),
    };

    let res = user
        .call(contract.id(), "register_client")
        .args_json(register)
        .transact()
        .await
        .unwrap();
    println!("res: {:?}", res);

    let account_id: AccountId = serde_json::from_slice(
        user.view(contract.id(), "get_account_id")
            .args_json(GetAccountId { client_type })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    assert_eq!(&account_id, lc.id());
    println!("[ + ] `register_client`: Client successfully registered");
}

async fn create_client(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    client_type: String,
) {
    let current_block = sandbox.view_block().await.unwrap();
    sleep(Duration::from_secs(2));
    let lc_block = sandbox
        .next_light_client_block(current_block.hash())
        .await
        .unwrap();
    let height = lc_block.inner_lite.height;
    let create = CreateClient {
        client_type: client_type.clone(),
        client_state: borsh::to_vec(&ClientState {
            latest_height: height - 1,
            ibc_account_id: ibc_contract.id().clone(),
            // TODO(aeryz): this is only valid in this sandboxed environment where the validator set is not changing. For a real environment,
            // the relayer must read the block producers using another endpoint.
            initial_block_producers: lc_block.next_bps.map(convert_block_producers),
        })
        .unwrap(),
        consensus_state: borsh::to_vec(&ConsensusState {
            state: convert_block_header_inner(lc_block.inner_lite),
            chunk_prev_state_root: CryptoHash(
                sandbox
                    .view_block()
                    .block_height(height)
                    .await
                    .unwrap()
                    .chunks()[0]
                    .prev_state_root
                    .0,
            ),
        })
        .unwrap(),
    };
    let res = user
        .call(ibc_contract.id(), "create_client")
        .args_json(create)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap();

    assert!(res.receipt_failures().is_empty());
    println!("[ + ] `create_client`: Client of type {client_type} created successfully");
}

async fn test_open_connection_starting_from_init(user: &Account, contract: &Contract) {
    let open_init = ConnectionOpenInit {
        client_id: "cometbls-1".into(),
        counterparty: connection_handshake::Counterparty {
            client_id: "08-wasm-0".to_string().validate().unwrap(),
            connection_id: "".into(),
            prefix: MerklePrefix {
                key_prefix: b"ibc".into(),
            },
        },
        version: DEFAULT_IBC_VERSION[0].clone(),
        delay_period: 0,
    };

    println!("calling connection open init");
    let res = user
        .call(contract.id(), "connection_open_init")
        .args_json(open_init)
        .transact()
        .await
        .unwrap();
    println!("connection open init res: {:?}", res);

    let connection_end_bytes: Vec<u8> = serde_json::from_slice(
        user.view(contract.id(), "get_commitment")
            .args_json(GetCommitment {
                key: "connections/connection-1".into(),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    let connection_end =
        connection_handshake::ConnectionEnd::decode_as::<Proto>(&connection_end_bytes).unwrap();

    assert_eq!(
        connection_handshake::ConnectionEnd {
            client_id: "cometbls-1".to_string().validate().unwrap(),
            versions: DEFAULT_IBC_VERSION.clone(),
            state: connection::state::State::Init,
            counterparty: connection_handshake::Counterparty {
                client_id: "08-wasm-0".to_string().validate().unwrap(),
                connection_id: "".into(),
                prefix: DEFAULT_MERKLE_PREFIX.clone()
            },
            delay_period: 0
        },
        connection_end
    );

    println!("Connection end: {connection_end:?}");

    let open_ack = ConnectionOpenAck {
        connection_id: "connection-1".to_string(),
        version: DEFAULT_IBC_VERSION[0].clone(),
        counterparty_connection_id: "connection-100".to_string(),
        connection_end_proof: vec![1, 2, 3],
        proof_height: Height {
            revision_number: 0,
            revision_height: 120,
        },
    };

    println!("calling connection open ack");
    let res = user
        .call(contract.id(), "connection_open_ack")
        .args_json(open_ack)
        .transact()
        .await
        .unwrap();

    println!("connectionopenack res: {res:?}");

    let connection_end_bytes: Vec<u8> = serde_json::from_slice(
        user.view(contract.id(), "get_commitment")
            .args_json(GetCommitment {
                key: "connections/connection-1".into(),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    let connection_end =
        connection_handshake::ConnectionEnd::decode_as::<Proto>(&connection_end_bytes).unwrap();
    assert_eq!(
        connection_handshake::ConnectionEnd {
            client_id: "cometbls-1".to_string().validate().unwrap(),
            versions: DEFAULT_IBC_VERSION.clone(),
            state: connection::state::State::Open,
            counterparty: connection_handshake::Counterparty {
                client_id: "08-wasm-0".to_string().validate().unwrap(),
                connection_id: "connection-100".into(),
                prefix: DEFAULT_MERKLE_PREFIX.clone()
            },
            delay_period: 0
        },
        connection_end
    );

    println!("[ + ] `test_open_connection_starting_from_init`: Connection opened.");
}

async fn test_open_channel_starting_from_init(
    user: &Account,
    contract: &Contract,
    ibc_app: &Contract,
) {
    let port_id = ibc_app.id().to_string().validate().unwrap();
    let channel_init = ChannelOpenInit {
        connection_hops: vec!["connection-1".to_string().validate().unwrap()],
        port_id: port_id.clone(),
        counterparty: channel::counterparty::Counterparty {
            port_id: "transfer".to_string().validate().unwrap(),
            channel_id: "".into(),
        },
        version: "ucs01".into(),
    };

    println!("calling channel open init");
    let res = user
        .call(contract.id(), "channel_open_init")
        .gas(Gas::from_gas(300000000000000))
        .args_json(channel_init)
        .transact()
        .await
        .unwrap();
    println!("channel open init res: {:?}", res);

    let channel_end_bytes: Vec<u8> = serde_json::from_slice(
        user.view(contract.id(), "get_commitment")
            .args_json(GetCommitment {
                key: format!(
                    "channelEnds/ports/{}/channels/channel-1",
                    port_id.to_string()
                ),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    let channel = Channel::decode_as::<Proto>(&channel_end_bytes).unwrap();

    assert_eq!(
        Channel {
            state: channel::state::State::Init,
            ordering: channel::order::Order::Unordered,
            counterparty: channel::counterparty::Counterparty {
                port_id: "transfer".to_string().validate().unwrap(),
                channel_id: "".into()
            },
            connection_hops: vec!["connection-1".to_string().validate().unwrap()],
            version: "ucs01".into()
        },
        channel
    );

    let channel_ack = ChannelOpenAck {
        channel_id: "channel-1".to_string().validate().unwrap(),
        port_id: port_id.clone(),
        counterparty_channel_id: "channel-100".into(),
        counterparty_version: "ucs01".into(),
        proof_try: vec![1, 2, 3],
        proof_height: Height {
            revision_number: 0,
            revision_height: 100,
        },
    };

    println!("calling channel open ack");
    let res = user
        .call(contract.id(), "channel_open_ack")
        .gas(Gas::from_gas(300000000000000))
        .args_json(channel_ack)
        .transact()
        .await
        .unwrap();
    println!("channel open ack res: {:?}", res);

    let channel_end_bytes: Vec<u8> = serde_json::from_slice(
        user.view(contract.id(), "get_commitment")
            .args_json(GetCommitment {
                key: format!(
                    "channelEnds/ports/{}/channels/channel-1",
                    port_id.to_string()
                ),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    let channel = Channel::decode_as::<Proto>(&channel_end_bytes).unwrap();

    assert_eq!(
        Channel {
            state: channel::state::State::Open,
            ordering: channel::order::Order::Unordered,
            counterparty: channel::counterparty::Counterparty {
                port_id: "transfer".to_string().validate().unwrap(),
                channel_id: "channel-100".into()
            },
            connection_hops: vec!["connection-1".to_string().validate().unwrap()],
            version: "ucs01".into()
        },
        channel
    );

    println!("[ + ] - `test_open_channel_starting_from_init`: Channel opened.");
}
