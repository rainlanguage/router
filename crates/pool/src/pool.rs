use alloy_sol_types::SolValue;
use std::collections::{BTreeMap, BTreeSet};
use alloy_primitives::{keccak256, Address, U256};

/// Represents a type for list of pool addresses
pub type PoolList = BTreeSet<Address>;

/// Represents a type for a map of pool addresses paired with an evm chain id
pub type PoolMap = BTreeMap<u64, ChainPools>;

/// Holds pool addresses of different types (such as univ2 protocol, univ3 protocol, etc) on an evm chain
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChainPools {
    pub univ2: PoolList,
    pub univ3: PoolList,
}

impl ChainPools {
    /// ref getter of pool list of the given type
    pub fn get_list(&self, pool_type: PoolType) -> &PoolList {
        match pool_type {
            PoolType::UniV2 => &self.univ2,
            PoolType::UniV3 => &self.univ3,
        }
    }

    /// mut ref getter of pool list of the given type
    pub fn get_list_mut(&mut self, pool_type: PoolType) -> &mut PoolList {
        match pool_type {
            PoolType::UniV2 => &mut self.univ2,
            PoolType::UniV3 => &mut self.univ3,
        }
    }
}

// Determines type of a pool, uniswapv2 , uniswapv3, etc
#[derive(Copy, Clone, Debug)]
pub enum PoolType {
    UniV2,
    UniV3,
}

// UniswapV3 protocol pool fee options
#[derive(Copy, Clone, Debug)]
#[repr(u64)]
pub enum UniV3Fee {
    // 0.01%
    LOWEST = 100,
    // 0.05%
    LOW = 500,
    // 0.3%
    MEDIUM = 3000,
    // 1%
    HIGH = 10000,
}

pub fn sort_address(addresses: [Address; 2]) -> [Address; 2] {
    if addresses[0] < addresses[1] {
        [addresses[0], addresses[1]]
    } else {
        [addresses[1], addresses[0]]
    }
}

/// Generates a uniswap protocl pool address using create2, given 2 tokens
/// addresses, a factory address and initCodeHash, optionally pool fee can
/// be given to generate address based on univ3 protocol, if no fee is given
/// the generated address will based on univ2 protocol
pub fn uni_pool_address(
    factory: Address,
    token1: Address,
    token2: Address,
    init_code_hash: U256,
    fee: Option<UniV3Fee>,
) -> Address {
    let [t1, t2] = sort_address([token1, token2]);
    let salt = if let Some(fee) = fee {
        keccak256((t1, t2, U256::from(fee as u64)).abi_encode())
    } else {
        keccak256((t1, t2).abi_encode_packed())
    };
    factory.create2(salt, init_code_hash.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex::FromHex;

    #[test]
    fn test_sort_addresses() -> anyhow::Result<()> {
        let address1 = Address::from_hex("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap();
        let address2 = Address::from_hex("0x3b9b5AD79cbb7649143DEcD5afc749a75F8e6C7F").unwrap();

        let expected = [address1, address2];

        let result = sort_address([address1, address2]);
        assert_eq!(result, expected);

        let result = sort_address([address2, address1]);
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_create2() -> anyhow::Result<()> {
        // univ3 protocol
        let univ3_pool_generated_address = uni_pool_address(
            Address::from_hex("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
            Address::from_hex("0x3b9b5AD79cbb7649143DEcD5afc749a75F8e6C7F").unwrap(),
            Address::from_hex("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(),
            U256::from_str_radix(
                "e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54",
                16,
            )
            .unwrap(),
            Some(UniV3Fee::HIGH),
        );
        let expected_address =
            Address::from_hex("0x72b236b8EB61B15833e514750b65b94a73D74A01").unwrap();
        assert_eq!(univ3_pool_generated_address, expected_address);

        // univ2 protocol
        let univ2_pool_generated_address = uni_pool_address(
            Address::from_hex("0x28b70f6Ed97429E40FE9a9CD3EB8E86BCBA11dd4").unwrap(),
            Address::from_hex("0x140d8d3649ec605cf69018c627fb44ccc76ec89f").unwrap(),
            Address::from_hex("0xff56eb5b1a7faa972291117e5e9565da29bc808d").unwrap(),
            U256::from_str_radix(
                "99e82d1f1ab2914f983fb7f2b987a3e30a55ad1fa8c38239d1f7c1a24fb93e3d",
                16,
            )
            .unwrap(),
            None,
        );
        let expected_address =
            Address::from_hex("0x87E0E33558c8e8EAE3c1E9EB276e05574190b48a").unwrap();
        assert_eq!(univ2_pool_generated_address, expected_address);

        Ok(())
    }

    #[test]
    fn test_chain_pools_getter() -> anyhow::Result<()> {
        let address1 = Address::from_hex("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap();
        let address2 = Address::from_hex("0x3b9b5AD79cbb7649143DEcD5afc749a75F8e6C7F").unwrap();

        let mut chain_pools = ChainPools::default();
        chain_pools.univ2.insert(address1);
        chain_pools.univ3.insert(address2);

        let mut expected_address1 = PoolList::new();
        expected_address1.insert(address1);

        let mut expected_address2 = PoolList::new();
        expected_address2.insert(address2);

        assert_eq!(chain_pools.get_list(PoolType::UniV2), &expected_address1);
        assert_eq!(chain_pools.get_list(PoolType::UniV3), &expected_address2);
        assert_eq!(
            chain_pools.get_list_mut(PoolType::UniV2),
            &mut expected_address1
        );
        assert_eq!(
            chain_pools.get_list_mut(PoolType::UniV3),
            &mut expected_address2
        );

        Ok(())
    }
}
