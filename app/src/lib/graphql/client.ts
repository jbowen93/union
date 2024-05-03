import {
  Client,
  cacheExchange,
  fetchExchange,
  debugExchange,
  subscriptionExchange
} from "@urql/svelte"
import { URLS } from "$lib/constants"
import { devtoolsExchange } from "@urql/devtools"
import { retryExchange } from "@urql/exchange-retry"
import { persistedExchange } from "@urql/exchange-persisted"
import { createClient as createWSClient, type SubscribePayload } from "graphql-ws"
import type { TadaPersistedDocumentNode } from "gql.tada"

const wsClient = createWSClient({
  url: URLS.GRAPHQL_WSS,
  shouldRetry: () => true
})

export const graphqlClient = new Client({
  url: URLS.GRAPHQL,
  exchanges: [
    devtoolsExchange,
    cacheExchange,
    persistedExchange({
      generateHash: async (_, document) => (document as TadaPersistedDocumentNode).documentId,
      preferGetForPersistedQueries: true,
      enforcePersistedQueries: true,
      enableForMutation: true,
      enableForSubscriptions: true
    }),
    fetchExchange,
    subscriptionExchange({
      forwardSubscription: operation => ({
        subscribe: sink => ({
          unsubscribe: wsClient.subscribe(operation as SubscribePayload, sink)
        })
      })
    }),
    retryExchange({
      randomDelay: true,
      maxDelayMs: 15_000,
      maxNumberAttempts: 2,
      initialDelayMs: 1_000,
      retryIf: error => !!error?.networkError?.message
    }),
    debugExchange
  ],
  fetchSubscriptions: true,

  fetchOptions: () => ({
    headers: {
      "X-Hasura-Admin-Secret": import.meta.env.PUBLIC_HASURA_ADMIN_SECRET ?? ""
    }
  })
})
