use std::io::stdin;

use vm::vm_client::VmClient;
use vm::{CreateReceiptRequest, ValidationRequest};
use zk_poc_core::{LedgerState, Transaction};
use zk_poc_methods::{INIT_ELF, INIT_ID, ISSUE_ELF, ISSUE_ID};
extern crate alloc;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;

use log::LevelFilter;
use serde::{Deserialize, Serialize};

pub mod vm {
    tonic::include_proto!("vm");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = VmClient::connect("http://[::1]:8080").await?;

    let mut addresses = BTreeMap::new();
    addresses.insert(String::from("A"), 1000);
    let ledger_maintainer_state = LedgerState {
        addresses: addresses,
        transfer_count: 0,
    };
    loop {
        println!("\nCreateReceiptRequest or ValidationRequest? (c/v)");
        let mut cv = String::new();
        stdin().read_line(&mut cv).unwrap();
        let cv = cv.trim();

        if cv == "c" {
            println!("Please provide transaction: ");
            println!("sender: ");
            let mut sender = String::new();
            stdin().read_line(&mut sender).unwrap();
            println!("receiver: ");
            let mut receiver = String::new();
            stdin().read_line(&mut receiver).unwrap();
            println!("tokens: ");
            let mut tokens = String::new();
            stdin().read_line(&mut tokens).unwrap();
            let tokens = tokens.trim().parse::<u32>().unwrap();
            let transaction = Transaction {
                sender: sender.trim().to_string(),
                receiver: receiver.trim().to_string(),
                tokens,
            };

            println!("Please provide signature: ");
            let mut signature = String::new();
            stdin().read_line(&mut signature).unwrap();

            let request = tonic::Request::new(CreateReceiptRequest {
                transaction: bincode::serialize(&transaction).unwrap(),
                signatures: bincode::serialize(&signature).unwrap(),
                ledger_state: bincode::serialize(&ledger_maintainer_state).unwrap(),
            });
            let response = client.create_receipt(request).await?;
            let response = response.into_inner();
            println!(
                "Got\n1) Receipt: '{:?}'\n2) new_ledger_state: '{:?}'\nfrom service",
                response.receipt, response.new_ledger_state
            );
        } else if cv == "v" {
            println!("Please provide receipt: ");
            let mut receipt = String::new();
            stdin().read_line(&mut receipt).unwrap();
            let receipt = receipt.trim();
            println!("Please provide method_id: ");
            let mut method_id = String::new();
            stdin().read_line(&mut method_id).unwrap();
            let method_id = method_id.trim().parse::<i32>().unwrap();

            let request = tonic::Request::new(ValidationRequest {
                receipt: bincode::deserialize(&receipt.as_bytes())?,
                method_id: method_id,
            });
            let response = client.validation(request).await?;
            println!("Got: '{}' from service", response.into_inner().valid);
        } else {
            break;
        }
    }
    Ok(())
}
