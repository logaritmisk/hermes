use crate::transaction::{Client, Transaction, Tx};

#[derive(Debug)]
pub struct Account {
    pub(crate) client: Client,
    pub(crate) available: f32,
    pub(crate) held: f32,
    pub(crate) locked: bool,
    pub(crate) ledger: Vec<Entry>,
}

impl Account {
    pub fn with_client(client: Client) -> Self {
        Self {
            client,
            available: 0.0,
            held: 0.0,
            locked: false,
            ledger: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn id(&self) -> u16 {
        self.client.id()
    }

    #[inline(always)]
    pub fn available(&self) -> f32 {
        self.available
    }

    #[inline(always)]
    pub fn held(&self) -> f32 {
        self.held
    }

    #[inline(always)]
    pub fn total(&self) -> f32 {
        self.available + self.held
    }

    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    #[inline(always)]
    pub(crate) fn find_transaction(&self, tx: Tx) -> Option<usize> {
        self.ledger.binary_search_by_key(&tx, |entry| entry.tx).ok()
    }

    pub(crate) fn push_transaction(&mut self, transaction: Transaction) {
        let entry = match transaction {
            Transaction::Deposit { tx, .. } => Entry {
                tx,
                transaction,
                disputed: false,
            },
            _ => return,
        };

        self.ledger.push(entry);
    }
}

// Keeps track on transaction and if they are under dispute. For now we only support dispute on deposit
// transactions, but we keep the complete transaction incase we want to add support for withdrawals as well.
#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) tx: Tx,
    pub(crate) transaction: Transaction,
    pub(crate) disputed: bool,
}
