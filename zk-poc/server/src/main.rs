use tonic::{transport::Server, Request, Response, Status};
use vm::{CreateReceiptRequest, ValidationRequest, CreateReceiptResponse, ValidationResponse,
         vm_server::{Vm, VmServer}};
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::{Prover, Receipt, Result};
use zk_poc_core:: {
    Transaction, InitializeLedgerCommit, IssueTransactionCommit, IssueTransactionParams, IssueTransactionResult, LedgerState,
};
use zk_poc_methods::{INIT_ELF, INIT_ID, ISSUE_ELF, ISSUE_ID};
extern crate alloc;
use alloc::{string::String};
use log::LevelFilter;
use serde::{Serialize, Deserialize};



// Add the trait to let this structure can be deserialized from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct InitMessage {
    receipt: Receipt,
}


pub mod vm {
    tonic::include_proto!("vm");
}


impl InitMessage {
    pub fn get_state(&self) -> Result<InitializeLedgerCommit> {
        Ok(from_slice(&self.receipt.journal).unwrap())
    }

    pub fn verity(&self) -> Result<bool> {
        match self.receipt.verify(INIT_ID) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn get_receipt(&self) -> Receipt {
        self.receipt.clone()
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

    pub fn verity(&self) -> Result<bool> {
        match self.receipt.verify(ISSUE_ID) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn get_receipt(&self) -> Receipt {
        self.receipt.clone()
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

    pub fn get_state(&self) -> LedgerState {
        self.state.clone()
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "[::1]:8080".parse().unwrap();
    let vm_service = VMService::default();
    env_logger::builder().filter_level(LevelFilter::Info).init();

    Server::builder().add_service(VmServer::new(vm_service))
        .serve(address)
        .await?;
    Ok(())
        
}

#[derive(Debug, Default)]
pub struct VMService {}

#[tonic::async_trait]
impl Vm for VMService {
    async fn create_receipt(&self, request: Request<CreateReceiptRequest>) -> Result<Response<CreateReceiptResponse>, Status> {
        let r = request.into_inner();

        // Get the transaction, signatures, and ledger_state from the request
        let transaction: Transaction = bincode::deserialize(r.transaction.as_slice()).unwrap();
        let _signatures: String = bincode::deserialize(r.signatures.as_slice()).unwrap();
        let ledger_state: LedgerState = bincode::deserialize(r.ledger_state.as_slice()).unwrap();
        
        // Create a new ledger maintainer with the ledger state
        let mut ledger_maintainer = LedgerMaintainer::new(ledger_state);
        
        // Issue the transaction
        let transaction_message = ledger_maintainer.issue(&transaction).unwrap();
        let receipt = transaction_message.get_receipt();

        return Ok(Response::new(vm::CreateReceiptResponse { 
            receipt: bincode::serialize(&receipt).unwrap(),
            new_ledger_state: bincode::serialize(&ledger_maintainer.get_state()).unwrap(),
        }));
    }
    async fn validation(&self, request: Request<ValidationRequest>) -> Result<Response<ValidationResponse>, Status> {
        let r = request.into_inner();

        // Get the receipt and the method_id from the request
        let receipt: Receipt = bincode::deserialize(r.receipt.as_slice()).unwrap();
        let method_id: i32 = r.method_id;

        // if method_id is 1 then it is the init method, else it is the issue method
        if method_id == 1 {
            // Verify the receipt
            let valid = match receipt.verify(INIT_ID) {
                Ok(_) => true,
                Err(_) => false,
            };

            return Ok(Response::new(vm::ValidationResponse { valid }));
        } else if method_id == 2 {
            // Verify the receipt
            let valid = match receipt.verify(ISSUE_ID) {
                Ok(_) => true,
                Err(_) => false,
            };

            return Ok(Response::new(vm::ValidationResponse { valid }));
        } else {
            return Ok(Response::new(vm::ValidationResponse { valid: false }));
        }
    }
}
