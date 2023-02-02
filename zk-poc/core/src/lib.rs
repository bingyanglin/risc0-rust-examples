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

#![cfg_attr(not(test), no_std)]

use risc0_zkp::core::sha::Digest;
use serde::{Deserialize, Serialize};

// use std::collections::hash_map::HashMap;
extern crate alloc;
use alloc::{string::String};
use alloc::collections::btree_map::BTreeMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LedgerState {
    pub addresses: BTreeMap<String, u32>,
    pub transfer_count: u32
}


impl LedgerState {
    pub fn transfer(&mut self, receiver: &String, sender: &String, tokens: u32) -> bool{
        let mut transfer_counted = false;
        if let Some(s) = self.addresses.get_mut(sender) {
            if *s >= tokens {
                *s -= tokens;
                if let Some(r) = self.addresses.get_mut(receiver) {
                    *r += tokens
                } else {
                    self.addresses.insert(String::from(receiver), tokens);
                }
                self.transfer_count += 1;
                transfer_counted = true;
            }   
        }
        transfer_counted
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InitializeLedgerCommit {
    pub state: Digest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Transaction {
    pub receiver: String,
    pub sender: String,
    pub tokens: u32,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IssueTransactionCommit {
    pub old_state: Digest,
    pub new_state: Digest,
    pub receiver: String,
    pub sender: String,
    pub tokens: u32,
    pub transfer_counted: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IssueTransactionParams {
    pub state: LedgerState,
    pub transaction: Transaction,
}


#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IssueTransactionResult {
    pub state: LedgerState,
    pub transfer_counted: bool,
    pub tokens: u32,
}

impl IssueTransactionParams {
    pub fn new(state: LedgerState, transaction: Transaction) -> Self {
        IssueTransactionParams {
            state: state,
            transaction: transaction,
        }
    }

    pub fn process(&self) -> IssueTransactionResult {
        let mut state = self.state.clone();
        let transfer_counted = state.transfer(&self.transaction.receiver, &self.transaction.sender, self.transaction.tokens);
        IssueTransactionResult {
            state: state,
            transfer_counted: transfer_counted,
            tokens: self.transaction.tokens,
        }
    }
}