use alloy_sol_types::SolValue;
use alloy_primitives::{keccak256, Address, U256};

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
pub fn uni_pool_addrerss(
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
    fn test_create2() -> anyhow::Result<()> {
        // univ3 protocol
        let univ3_pool_generated_address = uni_pool_addrerss(
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
        let univ2_pool_generated_address = uni_pool_addrerss(
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
}
