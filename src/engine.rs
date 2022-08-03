use std::collections::HashMap;

use crate::account::Account;
use crate::error::Error;
use crate::transaction::{Client, Transaction};

#[derive(Default, Debug)]
pub struct Engine {
    accounts: HashMap<Client, Account>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    pub fn apply(&mut self, transaction: Transaction) -> Result<(), Error> {
        match transaction {
            Transaction::Deposit { client, amount, .. } => {
                let account = self.get_or_create_account(client);

                // ASSUMPTION: if the account is locked, do not allow any new transaction
                if account.is_locked() {
                    return Err(Error::AccountLocked);
                }

                account.available += amount;
                account.push_transaction(transaction);
            }
            Transaction::Withdrawal { client, amount, .. } => {
                // TODO: if there is no account, maybe we should just ignore this transaction instead of creating a empty account?
                let account = self.get_or_create_account(client);

                // ASSUMPTION: if the account is locked, do not allow any new transaction
                if account.is_locked() {
                    return Err(Error::AccountLocked);
                }

                if account.available() < amount {
                    return Err(Error::InsufficientFunds);
                }

                account.available -= amount;
                account.push_transaction(transaction);
            }
            Transaction::Dispute { client, tx } => {
                // TODO: if there is no account, maybe we should just ignore this transaction instead of creating a empty account?
                let account = self.get_or_create_account(client);

                // ASSUMPTION: if the account is locked, do not allow any new transaction
                if account.is_locked() {
                    return Err(Error::AccountLocked);
                }

                let idx = account
                    .find_transaction(tx)
                    .ok_or(Error::TransactionNotFound)?;

                let entry = &mut account.ledger[idx];

                // TODO: add a error case `TransactionAlreadyUnderDispute`?
                if !entry.disputed {
                    if let Transaction::Deposit { amount, .. } = entry.transaction {
                        account.available -= amount;
                        account.held += amount;

                        entry.disputed = true;
                    }
                }
            }
            Transaction::Resolve { client, tx } => {
                // TODO: if there is no account, maybe we should just ignore this transaction instead of creating a empty account?
                let account = self.get_or_create_account(client);

                // ASSUMPTION: if the account is locked, do not allow any new transaction
                if account.is_locked() {
                    return Err(Error::AccountLocked);
                }

                let idx = account
                    .find_transaction(tx)
                    .ok_or(Error::TransactionNotFound)?;

                let entry = &mut account.ledger[idx];

                // the referenced transaction needs to be under dispute for a resolve to be valid
                if !entry.disputed {
                    return Err(Error::TransactionNotDisputed);
                }

                if let Transaction::Deposit { amount, .. } = entry.transaction {
                    account.available += amount;
                    account.held -= amount;
                }

                entry.disputed = false;
            }
            Transaction::Chargeback { client, tx } => {
                // TODO: if there is no account, maybe we should just ignore this transaction instead of creating a empty account?
                let account = self.get_or_create_account(client);

                // ASSUMPTION: if the account is locked, do not allow any new transaction
                if account.is_locked() {
                    return Err(Error::AccountLocked);
                }

                let idx = account
                    .find_transaction(tx)
                    .ok_or(Error::TransactionNotFound)?;

                let entry = &mut account.ledger[idx];

                // the referenced transaction needs to be under dispute for a chargeback to be valid
                if !entry.disputed {
                    return Err(Error::TransactionNotDisputed);
                }

                if let Transaction::Deposit { amount, .. } = entry.transaction {
                    account.held -= amount;

                    // as soon as a chargeback is made on an account, lock it!
                    account.locked = true;
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn get_or_create_account(&mut self, client: Client) -> &mut Account {
        self.accounts
            .entry(client)
            .or_insert_with(|| Account::with_client(client))
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction::{Client, Transaction, Tx};

    use super::*;

    #[test]
    fn test_engine_deposit() {
        let mut engine = Engine::new();

        let client = Client::from(1);

        let _ = engine.apply(Transaction::Deposit {
            client,
            tx: Tx::from(1),
            amount: 1.0,
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 1.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 1.0);
        assert!(!account.is_locked());

        let _ = engine.apply(Transaction::Deposit {
            client,
            tx: Tx::from(2),
            amount: 1.5,
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 2.5);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 2.5);
        assert!(!account.is_locked());
    }

    #[test]
    fn test_engine_withdraw() {
        let mut engine = Engine::new();

        let client = Client::from(1);

        let _ = engine.apply(Transaction::Deposit {
            client,
            tx: Tx::from(1),
            amount: 1.0,
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 1.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 1.0);
        assert!(!account.is_locked());

        let _ = engine.apply(Transaction::Withdrawal {
            client,
            tx: Tx::from(2),
            amount: 2.0,
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 1.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 1.0);
        assert!(!account.is_locked());
    }

    #[test]
    fn test_engine_dispute() {
        let mut engine = Engine::new();

        let client = Client::from(1);

        let _ = engine.apply(Transaction::Deposit {
            client,
            tx: Tx::from(1),
            amount: 1.0,
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 1.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 1.0);
        assert!(!account.is_locked());

        let _ = engine.apply(Transaction::Dispute {
            client,
            tx: Tx::from(1),
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 0.0);
        assert_eq!(account.held(), 1.0);
        assert_eq!(account.total(), 1.0);
        assert!(!account.is_locked());
    }

    #[test]
    fn test_engine_resolve() {
        let mut engine = Engine::new();

        let client = Client::from(1);

        let _ = engine.apply(Transaction::Deposit {
            client,
            tx: Tx::from(1),
            amount: 1.0,
        });

        let _ = engine.apply(Transaction::Deposit {
            client,
            tx: Tx::from(2),
            amount: 2.0,
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 3.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 3.0);
        assert!(!account.is_locked());

        let _ = engine.apply(Transaction::Dispute {
            client,
            tx: Tx::from(1),
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 2.0);
        assert_eq!(account.held(), 1.0);
        assert_eq!(account.total(), 3.0);
        assert!(!account.is_locked());

        let _ = engine.apply(Transaction::Resolve {
            client,
            tx: Tx::from(1),
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 3.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 3.0);
        assert!(!account.is_locked());
    }

    #[test]
    fn test_engine_chargeback() {
        let mut engine = Engine::new();

        let client = Client::from(1);

        let _ = engine.apply(Transaction::Deposit {
            client,
            tx: Tx::from(1),
            amount: 1.0,
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 1.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 1.0);
        assert!(!account.is_locked());

        let _ = engine.apply(Transaction::Dispute {
            client,
            tx: Tx::from(1),
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 0.0);
        assert_eq!(account.held(), 1.0);
        assert_eq!(account.total(), 1.0);
        assert!(!account.is_locked());

        let _ = engine.apply(Transaction::Chargeback {
            client,
            tx: Tx::from(1),
        });

        let account = &engine.accounts[&client];

        assert_eq!(account.available(), 0.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 0.0);
        assert!(account.is_locked());
    }
}
