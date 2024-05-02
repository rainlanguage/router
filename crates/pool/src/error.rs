use thiserror::Error;
use alloy_ethers_typecast::transaction::ReadableClientError;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ClientError(#[from] ReadableClientError),

    #[error(transparent)]
    AlloySolTypesError(#[from] alloy_sol_types::Error),

    #[error("Multicall item failed")]
    MulticallItemFailed(Vec<u8>),
}
