import { graphql } from "gql.tada"

export const allTransfersQueryDocument = graphql(/* GraphQL */ `
    query AllTransfersQuery @cached(ttl: 1) {
    v0_transfers(limit: 100, order_by: {source_timestamp: desc}) {
      sender
      source_chain_id
      source_transaction_hash
      receiver
      destination_chain_id
      assets
      source_timestamp
      forwards {
        chain {
          chain_id
        }
        port
        channel
        receiver
      }
    }
  }
`)

export const userTransfersQueryDocument = graphql(/* Graphql */ `
  query UserTransfersQuery($addr1: String!, $addr2: String!) @cached(ttl: 1) {
    v0_transfers(limit: 500, order_by: {source_timestamp: desc}, where: 
  {_or: [
  	{normalized_sender: {_eq: $addr1}}, 
  	{normalized_receiver: {_eq: $addr1}}, 
  	{normalized_sender: {_eq: $addr2}}, 
    {normalized_receiver: {_eq: $addr2}}
  ]}) {
      sender
      normalized_sender
      source_chain_id
      source_connection_id
      source_channel_id
      receiver
      normalized_receiver
      destination_chain_id
      destination_connection_id
      destination_channel_id
      assets
      source_timestamp
      destination_timestamp
    }
  }
`)

export const transfersBySourceHashBaseQueryDocument = graphql(/* GraphQL */ `
query TransfersBySourceHashBase($source_transaction_hash: String!) @cached(ttl: 1) {
  v0_transfers(where: {source_transaction_hash: {_eq: $source_transaction_hash}}) {
    sender
    normalized_sender
    source_chain_id
    source_connection_id
    source_channel_id
    source_transaction_hash
    receiver
    normalized_receiver
    destination_chain_id
    destination_connection_id
    destination_channel_id
    assets
    source_timestamp
    destination_timestamp
    forwards_2 {
      chain {
        chain_id
      }
      source_connection_id
      source_channel_id
      destination_connection_id
      destination_channel_id
      channel
      receiver
    }
  }
}
`)

export const transfersBySourceHashTracesAndHopsQueryDocument = graphql(/* GraphQL */ `
query TransfersBySourceHashTracesAndHops($source_transaction_hash: String!) @cached(ttl: 1) {
  v0_transfers(where: {source_transaction_hash: {_eq: $source_transaction_hash}}) {
    traces(order_by: {timestamp: asc}) {
      timestamp
      chain {
        chain_id
      }
      type
      transaction_hash
      height
    }
    hop {
      traces(order_by: {timestamp: asc}) {
        timestamp
        chain {
          chain_id
        }
        type
        transaction_hash
        height
      }
    }
  }
}
`)
