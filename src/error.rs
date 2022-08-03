use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("account locked")]
    AccountLocked,
    #[error("transaction not found")]
    TransactionNotFound,
    #[error("transaction not under dispute")]
    TransactionNotDisputed,
}
