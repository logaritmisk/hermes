#[derive(Debug)]
pub enum Transaction {
    Deposit { client: Client, tx: Tx, amount: f32 },
    Withdrawal { client: Client, tx: Tx, amount: f32 },
    Dispute { client: Client, tx: Tx },
    Resolve { client: Client, tx: Tx },
    Chargeback { client: Client, tx: Tx },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Client(u16);

impl Client {
    pub fn id(&self) -> u16 {
        self.0
    }
}

impl From<u16> for Client {
    fn from(id: u16) -> Self {
        Client(id)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Tx(u32);

impl From<u32> for Tx {
    fn from(tx: u32) -> Self {
        Tx(tx)
    }
}
