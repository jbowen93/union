import { get, writable } from "svelte/store"
import { CHAIN, URLS } from "$lib/constants"
import { sleep } from "$lib/utilities/index.ts"
import type { ChainWalletStore } from "$lib/wallet/types"
import { GasPrice, SigningStargateClient } from "@cosmjs/stargate"

export const cosmosWalletsInformation = [
  {
    id: "leap",
    name: "leap",
    icon: "https://assets.leapwallet.io/logos/leap-cosmos-logo.svg",
    download: "https://www.leapwallet.io/download"
  },
  {
    id: "keplr",
    name: "keplr",
    icon: "https://assets-global.website-files.com/63eb7ddf41cf5b1c8fdfbc74/63fc1eaf76d6a3bd547b017c_Keplr_icon_ver.1.3_2.svg",
    download: "https://www.keplr.app/download"
  }
] as const

export type CosmosWalletId = (typeof cosmosWalletsInformation)[number]["id"]

const stored = localStorage.getItem("cosmos-config") || "{}"

function createCosmosStore(
  previousState: ChainWalletStore<"cosmos"> & {
    signingStargateClient?: SigningStargateClient
  } = (JSON.parse(stored) as ChainWalletStore<"cosmos">) || {
    chain: "cosmos",
    hoverState: "none",
    address: undefined,
    connectedWallet: undefined,
    connectionStatus: "disconnected",
    signingStargateClient: undefined
  }
) {
  const { subscribe, set, update } = writable(previousState)
  return {
    set,
    update,
    subscribe,
    connect: async (walletId: string) => {
      if (!walletId || (walletId !== "keplr" && walletId !== "leap")) return
      update(v => ({ ...v, connectionStatus: "connecting" }))
      if (!window[walletId]) {
        alert(`Please install ${walletId} wallet`)
        update(v => ({ ...v, connectionStatus: "disconnected" }))
        return
      }
      await window[walletId]?.enable(CHAIN.UNION.ID)
      const offlineSigner = window[walletId as "keplr"]?.getOfflineSigner(CHAIN.UNION.ID, {
        disableBalanceCheck: false
      })
      if (!offlineSigner) return console.error("[cosmos] No offline signer found")
      const accounts = await offlineSigner?.getAccounts()
      console.log("[cosmos] accounts", accounts)
      update(v => ({ ...v, connectionStatus: "connected" }))
      if (!accounts) return console.error("[cosmos] No accounts found")
      const [account] = accounts
      update(v => ({ ...v, address: account.address }))
      const stargateClient = await SigningStargateClient.connectWithSigner(
        URLS.UNION.RPC,
        offlineSigner,
        { gasPrice: GasPrice.fromString("0.025muno") }
      )
      update(v => ({ ...v, connectedWallet: walletId, signingStargateClient: stargateClient }))
      await sleep(2_000)
    },
    disconnect: async () => {
      const cosmosWalletId = get({ subscribe }).connectedWallet as CosmosWalletId
      console.log("[cosmos] cosmosDisconnectClick", get(cosmosStore))
      if (cosmosWalletId && cosmosWalletId === "keplr" && window[cosmosWalletId]) {
        await window[cosmosWalletId]?.disable([CHAIN.UNION.ID])
        update(v => ({ ...v, connectedWallet: "none", connectionStatus: "disconnected" }))
      }
      if (cosmosWalletId && cosmosWalletId === "leap" && window[cosmosWalletId]) {
        await window[cosmosWalletId]?.disconnect(CHAIN.UNION.ID)
        update(v => ({ ...v, connectedWallet: "none", connectionStatus: "disconnected" }))
      }
    }
  }
}

export const cosmosStore = createCosmosStore()
cosmosStore.subscribe(value => localStorage.setItem("cosmos-config", JSON.stringify(value)))
