import erc20abi from '$lib/abi/erc20.json';
import ibcAbi from '$lib/abi/ibc.json';

export const UNION_CHAIN_ID = 'union-testnet-3';
export const MUNO_ERC20_ADDRESS = '0x1ea17f35801d9d7c160f66603ac4c5bb59fcec19';
export const UCS01_EVM_ADDRESS = '0xd5DA8d1667227F0143Ded9d5f654e08CA5e3D3EB';
export const UCS01_UNION_ADDRESS =
	'union1ya4lhhrxx00nskcqlt8k0sggyl7akxdlhkc95ydme75gnjr0hmrs6wkhm6';

export const ERC20_CONTRACT_ABI = erc20abi.abi;
export const IBC_CONTRACT_ABI = ibcAbi.abi;
export const UCS01_UNION_SOURCE_CHANNEL = 'channel-16';
export const UCS01_SEPOLIA_SOURCE_CHANNEL = 'channel-6';
export const UCS01_SEPOLIA_PORT_ID = 'ucs01-relay-2';
