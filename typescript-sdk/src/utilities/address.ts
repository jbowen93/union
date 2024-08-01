import { isAddress, getAddress } from "viem"
import { fromBech32 } from "@cosmjs/encoding"
import type { HexAddress, Bech32Address } from "../types.ts"

export function truncateAddress({
  address,
  length = 6
}: {
  address: string
  length?: number
}) {
  return length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address
}

export const isValidEvmAddress = (address: unknown): address is HexAddress =>
  typeof address === "string" && isAddress(address) && getAddress(address) === address

export function isValidBech32Address(
  address: unknown,
  { expectedPrefixes }: { expectedPrefixes: ["union"] } = { expectedPrefixes: ["union"] }
): address is Bech32Address {
  if (typeof address !== "string") return false

  try {
    const { prefix, data } = fromBech32(address)
    if (expectedPrefixes && !expectedPrefixes.includes(prefix)) return false

    const size = data.length
    if ([20, 32].indexOf(size) === -1) return false

    return true
  } catch (error) {
    return false
  }
}
