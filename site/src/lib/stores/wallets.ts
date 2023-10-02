import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { Tendermint37Client } from '@cosmjs/tendermint-rpc';
import type { SigningStargateClient } from '@cosmjs/stargate';
import type { AccountData, Coin } from '@cosmjs/amino';
import type { ApolloClient, InMemoryCache, NormalizedCacheObject } from '@apollo/client';

export const tendermintClient: Writable<Tendermint37Client | null> = writable(null);
export const stargateClient: Writable<SigningStargateClient | null> = writable(null);
export const unionAccount: Writable<AccountData | null> = writable(null);
export const unionBalance: Writable<Coin | null> = writable(null);
export const apolloClient: Writable<ApolloClient<NormalizedCacheObject> | null> = writable(null);
export const ethersProvider: Writable<any | null> = writable(null);
