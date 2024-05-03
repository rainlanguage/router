use once_cell::sync::Lazy;
use alloy_primitives::Address;
use crate::{error::Error, pool::ChainPools};
use super::pool::{PoolList, PoolMap, PoolType};
use std::{
    collections::BTreeSet,
    sync::{Arc, RwLock},
};

/// Holds addresses that are not an actual pool per evm chain
pub static POOL_BLACKLIST: Lazy<Arc<RwLock<PoolMap>>> =
    Lazy::new(|| Arc::new(RwLock::new(PoolMap::default())));

/// Holds addresses that are already known to be an actual pool per evm chain
pub static POOL_WHITELIST: Lazy<Arc<RwLock<PoolMap>>> =
    Lazy::new(|| Arc::new(RwLock::new(PoolMap::default())));

/// adds addresses to blacklist by the given chain id and pool type
pub fn add_to_blacklist<'a, 'b>(
    list: impl IntoIterator<Item = &'a Address>,
    chain_id: u64,
    pool_type: PoolType,
) -> Result<(), Error<'b>> {
    let mut blacklist_map = POOL_BLACKLIST.write()?;
    if let Some(blacklist) = blacklist_map.get_mut(&chain_id) {
        blacklist.get_list_mut(pool_type).extend(list);
    } else {
        let mut chain_pools = ChainPools::default();
        chain_pools.get_list_mut(pool_type).extend(list);
        blacklist_map.insert(chain_id, chain_pools);
    }
    Ok(())
}

/// removes addresses from blacklist by the given chain id and pool type
pub fn remove_from_blacklist<'a, 'b>(
    list: impl IntoIterator<Item = &'a Address>,
    chain_id: u64,
    pool_type: PoolType,
) -> Result<(), Error<'b>> {
    let mut blacklist_map = POOL_BLACKLIST.write()?;
    if let Some(blacklist) = blacklist_map.get_mut(&chain_id) {
        let mut iter = list.into_iter();
        blacklist
            .get_list_mut(pool_type)
            .retain(|v| iter.all(|e| e != v))
    }
    Ok(())
}

/// adds addresses to whitelist by the given chain id and pool type
pub fn add_to_whitelist<'a, 'b>(
    list: impl IntoIterator<Item = &'a Address>,
    chain_id: u64,
    pool_type: PoolType,
) -> Result<(), Error<'b>> {
    let mut whitelist_map = POOL_WHITELIST.write()?;
    if let Some(whitelist) = whitelist_map.get_mut(&chain_id) {
        whitelist.get_list_mut(pool_type).extend(list);
    } else {
        let mut chain_pools = ChainPools::default();
        chain_pools.get_list_mut(pool_type).extend(list);
        whitelist_map.insert(chain_id, chain_pools);
    }
    Ok(())
}

/// removes addresses from whitelist by the given chain id and pool type
pub fn remove_from_whitelist<'a, 'b>(
    list: impl IntoIterator<Item = &'a Address>,
    chain_id: u64,
    pool_type: PoolType,
) -> Result<(), Error<'b>> {
    let mut whitelist_map = POOL_WHITELIST.write()?;
    if let Some(whitelist) = whitelist_map.get_mut(&chain_id) {
        let mut iter = list.into_iter();
        whitelist
            .get_list_mut(pool_type)
            .retain(|v| iter.all(|e| e != v))
    }
    Ok(())
}

/// Filters out blacklisted addresses by the given addresses
pub fn filter_by_blacklist<'a, 'b>(
    list: impl IntoIterator<Item = &'a Address>,
    chain_id: u64,
    pool_type: PoolType,
) -> Result<PoolList, Error<'b>> {
    let mut filtered_list = PoolList::new();
    let blacklist_map = POOL_BLACKLIST.read()?;
    if let Some(blacklist) = blacklist_map.get(&chain_id) {
        let pool_typed_list = blacklist.get_list(pool_type);
        list.into_iter().for_each(|p| {
            if !pool_typed_list.contains(p) {
                filtered_list.insert(*p);
            }
        });
    }
    Ok(filtered_list)
}

/// Filters out whitelisted addresses by the given addresses
/// and retuns result + the list whitelisted addresses
pub fn filter_by_whitelist<'a, 'b>(
    list: impl IntoIterator<Item = &'a Address>,
    chain_id: u64,
    pool_type: PoolType,
) -> Result<(PoolList, PoolList), Error<'b>> {
    let mut filtered_list = PoolList::new();
    let mut intersection_list = PoolList::new();
    let whitelist_map = POOL_WHITELIST.read()?;
    if let Some(whitelist) = whitelist_map.get(&chain_id) {
        let pool_typed_list = whitelist.get_list(pool_type);
        for p in list {
            if !pool_typed_list.contains(p) {
                filtered_list.insert(*p);
            } else {
                intersection_list.insert(*p);
            }
        }
    }
    Ok((filtered_list, intersection_list))
}

/// Filters the given addresses by both known blacklist and whitelist
pub fn filter_all<'a, 'b>(
    list: impl IntoIterator<Item = &'a Address>,
    chain_id: u64,
    pool_type: PoolType,
) -> Result<(PoolList, PoolList), Error<'b>> {
    let empty_list = BTreeSet::new();

    let whitelist_map = POOL_WHITELIST.read()?;
    let whitelist = whitelist_map
        .get(&chain_id)
        .map_or(&empty_list, |v| v.get_list(pool_type));

    let blacklist_map = POOL_BLACKLIST.read()?;
    let blacklist = blacklist_map
        .get(&chain_id)
        .map_or(&empty_list, |v| v.get_list(pool_type));

    let mut filtered_list = PoolList::new();
    let mut intersection_whitelist = PoolList::new();
    for p in list {
        if !blacklist.contains(p) {
            if !whitelist.contains(p) {
                filtered_list.insert(*p);
            } else {
                intersection_whitelist.insert(*p);
            }
        }
    }
    Ok((filtered_list, intersection_whitelist))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use alloy_primitives::hex::FromHex;

    #[test]
    #[serial]
    fn test_add_remove_blacklist() -> anyhow::Result<()> {
        let address1 = Address::from_hex("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap();
        let address2 = Address::from_hex("0x3b9b5AD79cbb7649143DEcD5afc749a75F8e6C7F").unwrap();

        // test add_to_blacklist
        let list = vec![address1, address2];
        add_to_blacklist(&list, 1, PoolType::UniV2).unwrap();

        let mut expected_pool_list = PoolList::default();
        expected_pool_list.insert(address1);
        expected_pool_list.insert(address2);
        let mut expected = PoolMap::default();
        expected.insert(
            1,
            ChainPools {
                univ2: expected_pool_list,
                univ3: PoolList::default(),
            },
        );

        {
            let readable_pool_blacklist = POOL_BLACKLIST.read().unwrap().clone();
            assert_eq!(readable_pool_blacklist, expected);
        };

        // test remove _from_blacklist
        let list = vec![address1];
        remove_from_blacklist(&list, 1, PoolType::UniV2).unwrap();

        let mut expected_pool_list = PoolList::default();
        expected_pool_list.insert(address2);
        let mut expected = PoolMap::default();
        expected.insert(
            1,
            ChainPools {
                univ2: expected_pool_list,
                univ3: PoolList::default(),
            },
        );

        {
            let readable_pool_blacklist = POOL_BLACKLIST.read().unwrap().clone();
            assert_eq!(readable_pool_blacklist, expected);
        };

        // make sure to empty the POOL_BLACKLIST for following tests
        remove_from_blacklist(&vec![address2], 1, PoolType::UniV2).unwrap();

        Ok(())
    }

    #[test]
    #[serial]
    fn test_add_remove_whitelist() -> anyhow::Result<()> {
        let address1 = Address::from_hex("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap();
        let address2 = Address::from_hex("0x3b9b5AD79cbb7649143DEcD5afc749a75F8e6C7F").unwrap();
        let list = vec![address1, address2];

        add_to_whitelist(&list, 1, PoolType::UniV2).unwrap();

        let mut expected_pool_list = PoolList::default();
        expected_pool_list.insert(address1);
        expected_pool_list.insert(address2);
        let mut expected = PoolMap::default();
        expected.insert(
            1,
            ChainPools {
                univ2: expected_pool_list,
                univ3: PoolList::default(),
            },
        );

        {
            let readable_pool_whitelist = POOL_WHITELIST.read().unwrap().clone();
            assert_eq!(readable_pool_whitelist, expected);
        };

        let list = vec![address1];
        remove_from_whitelist(&list, 1, PoolType::UniV2).unwrap();

        let mut expected_pool_list = PoolList::default();
        expected_pool_list.insert(address2);
        let mut expected = PoolMap::default();
        expected.insert(
            1,
            ChainPools {
                univ2: expected_pool_list,
                univ3: PoolList::default(),
            },
        );

        {
            let readable_pool_whitelist = POOL_WHITELIST.read().unwrap().clone();
            assert_eq!(readable_pool_whitelist, expected);
        };

        // make sure to empty the POOL_BLACKLIST for following tests
        remove_from_whitelist(&vec![address2], 1, PoolType::UniV2).unwrap();

        Ok(())
    }

    #[test]
    #[serial]
    fn test_filter_by_blacklist() -> anyhow::Result<()> {
        let address1 = Address::from_hex("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap();
        let address2 = Address::from_hex("0x3b9b5AD79cbb7649143DEcD5afc749a75F8e6C7F").unwrap();

        add_to_blacklist(&vec![address1], 1, PoolType::UniV2).unwrap();

        let list = vec![address1, address2];
        let filtered_list = filter_by_blacklist(&list, 1, PoolType::UniV2).unwrap();

        let mut expected_pool_list = PoolList::default();
        expected_pool_list.insert(address2);

        assert_eq!(expected_pool_list, filtered_list);

        // make sure to empty the POOL_BLACKLIST for following tests
        remove_from_blacklist(&vec![address1], 1, PoolType::UniV2).unwrap();

        Ok(())
    }

    #[test]
    #[serial]
    fn test_filter_by_whitelist() -> anyhow::Result<()> {
        let address1 = Address::from_hex("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap();
        let address2 = Address::from_hex("0x3b9b5AD79cbb7649143DEcD5afc749a75F8e6C7F").unwrap();

        add_to_whitelist(&vec![address1], 1, PoolType::UniV2).unwrap();

        let list = vec![address1, address2];
        let filtered_list = filter_by_whitelist(&list, 1, PoolType::UniV2).unwrap();

        let mut expected_pool_list = PoolList::default();
        expected_pool_list.insert(address2);

        let mut expected_intersection_list = PoolList::default();
        expected_intersection_list.insert(address1);

        assert_eq!(expected_pool_list, filtered_list.0);
        assert_eq!(expected_intersection_list, filtered_list.1);

        // make sure to empty the POOL_BLACKLIST for following tests
        remove_from_whitelist(&vec![address1], 1, PoolType::UniV2).unwrap();

        Ok(())
    }

    #[test]
    #[serial]
    fn test_filter_all() -> anyhow::Result<()> {
        let address1 = Address::from_hex("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap();
        let address2 = Address::from_hex("0x3b9b5AD79cbb7649143DEcD5afc749a75F8e6C7F").unwrap();
        let address3 = Address::from_hex("0xff56eb5b1a7faa972291117e5e9565da29bc808d").unwrap();
        let address4 = Address::from_hex("0x87E0E33558c8e8EAE3c1E9EB276e05574190b48a").unwrap();

        add_to_whitelist(&vec![address1], 1, PoolType::UniV2).unwrap();
        add_to_blacklist(&vec![address2], 1, PoolType::UniV2).unwrap();

        let list = vec![address1, address2, address3, address4];
        let filtered_list = filter_all(&list, 1, PoolType::UniV2).unwrap();

        let mut expected_pool_list = PoolList::default();
        expected_pool_list.insert(address3);
        expected_pool_list.insert(address4);

        let mut expected_intersection_whitelist = PoolList::default();
        expected_intersection_whitelist.insert(address1);

        assert_eq!(expected_pool_list, filtered_list.0);
        assert_eq!(expected_intersection_whitelist, filtered_list.1);

        // make sure to empty the static lists for following tests
        remove_from_blacklist(&vec![address2], 1, PoolType::UniV2).unwrap();
        remove_from_whitelist(&vec![address1], 1, PoolType::UniV2).unwrap();

        Ok(())
    }
}
