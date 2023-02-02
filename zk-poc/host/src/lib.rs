// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::{Prover, Receipt, Result};
use zk_poc_core:: {
    Transaction, InitializeLedgerCommit, IssueTransactionCommit, IssueTransactionParams, IssueTransactionResult, LedgerState,
};
use zk_poc_methods::{INIT_ELF, INIT_ID, ISSUE_ELF, ISSUE_ID};




pub struct InitMessage {
    receipt: Receipt,
}

impl InitMessage {
    pub fn get_state(&self) -> Result<InitializeLedgerCommit> {
        Ok(from_slice(&self.receipt.journal).unwrap())
    }

    pub fn verify_and_get_commit(&self) -> Result<InitializeLedgerCommit> {
        self.receipt.verify(INIT_ID)?;
        self.get_state()
    }
}

pub struct IssueTransactionMessage {
    receipt: Receipt,
}

impl IssueTransactionMessage {
    pub fn get_commit(&self) -> Result<IssueTransactionCommit> {
        Ok(from_slice(&self.receipt.journal).unwrap())
    }

    pub fn verify_and_get_commit(&self) -> Result<IssueTransactionCommit> {
        self.receipt.verify(ISSUE_ID)?;
        self.get_commit()
    }
}


#[derive(Debug)]
pub struct LedgerMaintainer {
    state: LedgerState,
}

impl LedgerMaintainer {
    pub fn new(state: LedgerState) -> Self {
        LedgerMaintainer { state }
    }

    pub fn init(&self) -> Result<InitMessage> {
        log::info!("init");
        let mut prover = Prover::new(INIT_ELF, INIT_ID)?;
        let vec = to_vec(&self.state).unwrap();
        prover.add_input_u32_slice(vec.as_slice());
        let receipt = prover.run()?;
        Ok(InitMessage { receipt })
    }

    pub fn issue(&mut self, transaction: &Transaction) -> Result<IssueTransactionMessage> {
        log::info!("issue: {:?}", transaction);
        let params = IssueTransactionParams::new(self.state.clone(), transaction.clone());
        let mut prover = Prover::new(ISSUE_ELF, ISSUE_ID)?;
        let vec = to_vec(&params).unwrap();
        prover.add_input_u32_slice(vec.as_slice());
        let receipt = prover.run()?;
        let vec = prover.get_output_u32_vec()?;
        log::info!("{:?}", vec);
        let result = from_slice::<IssueTransactionResult>(&vec);
        log::info!("{:?}", result);
        self.state = result.unwrap().state.clone();
        Ok(IssueTransactionMessage { receipt })
    }
}




#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::{string::String};
    use alloc::collections::btree_map::BTreeMap;
    use log::LevelFilter;
    use super::*;

    #[ctor::ctor]
    fn init() {
        env_logger::builder().filter_level(LevelFilter::Info).init();
    }

    #[test]
    fn protocol() {
        let mut addresses = BTreeMap::new();
        addresses.insert(String::from("addressA"), 1000);
        let ledger_maintainer_state = LedgerState {
            addresses: addresses,
            transfer_count: 0,
        };

        let mut ledger_maintainer = LedgerMaintainer::new(ledger_maintainer_state);

        let transaction1 = Transaction {
            sender: String::from("addressA"),
            receiver: String::from("addressB"),
            tokens: 100,
        };
        
        let transaction2 = Transaction {
            sender: String::from("addressB"),
            receiver: String::from("addressC"),
            tokens: 50,
        };

        let init_msg = ledger_maintainer.init().unwrap();
        let transaction_msg1 = ledger_maintainer.issue(&transaction1).unwrap();
        let transaction_msg2 = ledger_maintainer.issue(&transaction2).unwrap();
        
        assert_eq!(ledger_maintainer.state.transfer_count, 2);

        let init_state = init_msg.verify_and_get_commit();
        let transaction_commit1 = transaction_msg1.verify_and_get_commit();
        let transaction_commit2 = transaction_msg2.verify_and_get_commit();
    

        log::info!("initial commit: {:?}", init_state);
        log::info!("transaction 1: {:?}", transaction1);
        log::info!("transaction 1 commit: {:?}", transaction_commit1);
        log::info!("transaction 2: {:?}", transaction2);
        log::info!("transaction 2 commit: {:?}", transaction_commit2);
    }
}
