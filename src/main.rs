use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader};
use std::process;

use csv::{ReaderBuilder, WriterBuilder};
use hermes::{Client, Engine, Transaction, Tx};

fn execute<R: io::Read>(reader: R) -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::new();

    let mut rdr = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(reader);

    // NOTE: Assume the order of columns always is the same
    for result in rdr.records() {
        // NOTE: I would like to use serde to deserialize the transaction, but due to https://github.com/BurntSushi/rust-csv/issues/278 we can't
        let result = result?;

        let transaction = match &result[0] {
            "deposit" => Transaction::Deposit {
                client: Client::from(result[1].trim().parse::<u16>()?),
                tx: Tx::from(result[2].trim().parse::<u32>()?),
                amount: result[3].trim().parse()?,
            },
            "withdrawal" => Transaction::Withdrawal {
                client: Client::from(result[1].trim().parse::<u16>()?),
                tx: Tx::from(result[2].trim().parse::<u32>()?),
                amount: result[3].trim().parse()?,
            },
            "dispute" => Transaction::Dispute {
                client: Client::from(result[1].trim().parse::<u16>()?),
                tx: Tx::from(result[2].trim().parse::<u32>()?),
            },
            "resolve" => Transaction::Resolve {
                client: Client::from(result[1].trim().parse::<u16>()?),
                tx: Tx::from(result[2].trim().parse::<u32>()?),
            },
            "chargeback" => Transaction::Chargeback {
                client: Client::from(result[1].trim().parse::<u16>()?),
                tx: Tx::from(result[2].trim().parse::<u32>()?),
            },
            _ => {
                panic!("invalid type of transaction");
            }
        };

        // TODO: Log errors to stderr
        let _ = engine.apply(transaction);
    }

    let mut wrt = WriterBuilder::new().from_writer(io::stdout());

    wrt.write_record(&["client", "available", "held", "total", "locked"])?;

    for account in engine.accounts() {
        wrt.write_record(&[
            format!("{}", account.id()),
            format!("{:.4}", account.available()),
            format!("{:.4}", account.held()),
            format!("{:.4}", account.total()),
            format!("{}", account.is_locked()),
        ])?;
    }

    wrt.flush()?;

    Ok(())
}

fn main() {
    let input = env::args().nth(1).expect("a file");

    let data = File::open(input)
        .map(BufReader::new)
        .expect("an opened file");

    if let Err(err) = execute(data) {
        println!("error running example: {}", err);

        process::exit(1);
    }
}
