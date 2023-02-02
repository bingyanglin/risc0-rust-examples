# ZK PoC (Step 0)

## RISC-0 Basics

- [Understanding the prover](https://www.risczero.com/docs/examples/starter)
- [RISC0-Rust-Examples](https://github.com/risc0/risc0-rust-examples/)
- [Starter Template](https://github.com/risc0/risc0-rust-starter)
- `Receipt`: Validity Proof (Journal + Seal)
  - The journal contains the public outputs of the computation.
  - The seal is an opaque cryptographic blob that attests to the integrity of the computation. It consists of merkle commitments and query data for an AIR-FRI STARK that includes a PLONK-based permutation argument.
- `Prover`

  - Take the ELF file and the ID
    - ELF: the binary file for `method` execution; ID: the hash of the ELF file
      - Each method is used to generate the `commit`

## Mockup

### core/src folder

- lib.rs (Define structures)
- The commit contain the `state`, whose type is `risc0_zkp::core::sha::Digest`

```rust
pub struct LedgerState {
    pub addresses: BTreeMap<String, u32>,
    pub transfer_count: u32
}


impl LedgerState {
    pub fn transfer(&mut self, receiver: &String, sender: &String, tokens: u32) -> bool{
        // Transfer tokens from a sender address to a receiver address
        // Return whether the transfer is successful or not
    }
}

pub struct InitializeLedgerCommit {
    pub state: Digest,
}

pub struct Transaction {
    pub receiver: String,
    pub sender: String,
    pub tokens: u32,
}

pub struct IssueTransactionCommit {
    pub old_state: Digest,
    pub new_state: Digest,
    pub receiver: String,
    pub sender: String,
    pub tokens: u32,
    pub transfer_counted: bool,
}

pub struct IssueTransactionParams {
    pub state: LedgerState,
    pub transaction: Transaction,
}


pub struct IssueTransactionResult {
    pub state: LedgerState,
    pub transfer_counted: bool,
    pub tokens: u32,
}

impl IssueTransactionParams {
    pub fn new(state: LedgerState, transaction: Transaction) -> Self {
        // Create a IssueTransactionParams
    }

    pub fn process(&self) -> IssueTransactionResult {
        // Transfer tokens from the sender address to the receiver address
        // Return the IssueTransactionResult
    }
}
```

### method folder

- guest/src/bin folder
  - init.rs
    - Generate the `InitializeVotingMachineCommit` by the RISC0 VM
  - issue.rs
    - Generate the `IssueTransactionCommit` by the RISC0 VM
- src folder
  - build.rs
    - `risc0_build::embed_methods()`

### host/src folder

- Contains the structures and methods for creating and verifying the receipt)
- lib.rs

  - `InitMessage` (contains Receipt)
    - `pub fn get_state(&self) -> Result<InitializeVotingMachineCommit>`
      - Get the state (receipt.journal)
    - `pub fn verify_and_get_commit(&self) -> Result<InitializeVotingMachineCommit>`
      - Verify the receipt by calling receipt.verify(), and return the commit
  - `IssueTransactionMessage` is similar to `InitMessage`
  - `LedgerMaintainer` (Generate the receipt by running the Prover)
    - `pub fn init(&self) -> Result<InitMessage>`
    - `pub fn issue(&mut self, ballot: &Transaction) -> Result<IssueTransactionMessage>`

- Testing
  - Create a new LedgerMaintainer
  - Prepare Transactions
  - Issue Transactions via the LedgerMaintainer, and get the IssueTransactionMessage
  - Verity IssueTransactionMessage
  - Check the transaction commits and the transfer_count
