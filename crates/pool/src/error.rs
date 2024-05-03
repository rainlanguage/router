use thiserror::Error;
use crate::pool::ChainPools;
use std::collections::BTreeMap;
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error(transparent)]
    StaticPoolListPoisonReadError(PoisonError<RwLockReadGuard<'a, BTreeMap<u64, ChainPools>>>),

    #[error(transparent)]
    StaticPoolListPoisonWriteError(PoisonError<RwLockWriteGuard<'a, BTreeMap<u64, ChainPools>>>),
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, BTreeMap<u64, ChainPools>>>> for Error<'a> {
    fn from(value: PoisonError<RwLockReadGuard<'a, BTreeMap<u64, ChainPools>>>) -> Self {
        Self::StaticPoolListPoisonReadError(value)
    }
}

impl<'a> From<PoisonError<RwLockWriteGuard<'a, BTreeMap<u64, ChainPools>>>> for Error<'a> {
    fn from(value: PoisonError<RwLockWriteGuard<'a, BTreeMap<u64, ChainPools>>>) -> Self {
        Self::StaticPoolListPoisonWriteError(value)
    }
}
