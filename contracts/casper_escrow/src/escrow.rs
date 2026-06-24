use odra::prelude::*;
use odra::casper_types::U512;

#[odra::odra_type]
pub struct Bounty {
    pub creator: Address,
    pub hunter: Address,
    pub verifier: Address,
    pub amount: U512,
    pub status: u8, // 0 = Locked, 1 = ProofSubmitted, 2 = Released, 3 = Refunded
}

#[odra::odra_error]
pub enum Error {
    BountyNotFound = 1,
    NotVerifier = 2,
    InvalidStatus = 3,
    ZeroDeposit = 4,
    NotCreatorOrVerifier = 5,
}

#[odra::module]
pub struct EscrowContract {
    bounties: Mapping<String, Bounty>,
}

#[odra::module]
impl EscrowContract {
    #[odra(init)]
    pub fn init(&mut self) {}

    #[odra(payable)]
    pub fn deposit(&mut self, task_id: String, hunter: Address, verifier: Address) {
        let amount = self.env().attached_value();
        if amount == U512::zero() {
            self.env().revert(Error::ZeroDeposit);
        }

        let bounty = Bounty {
            creator: self.env().caller(),
            hunter,
            verifier,
            amount,
            status: 0,
        };

        self.bounties.set(&task_id, bounty);
    }

    pub fn submit_proof(&mut self, task_id: String, _signature: String) {
        let mut bounty = self.get_bounty(&task_id);
        
        if bounty.status != 0 {
            self.env().revert(Error::InvalidStatus);
        }

        bounty.status = 1;
        self.bounties.set(&task_id, bounty);
    }

    pub fn release(&mut self, task_id: String) {
        let mut bounty = self.get_bounty(&task_id);
        let caller = self.env().caller();

        if caller != bounty.verifier {
            self.env().revert(Error::NotVerifier);
        }

        if bounty.status != 1 {
            self.env().revert(Error::InvalidStatus);
        }

        self.env().transfer_tokens(&bounty.hunter, &bounty.amount);

        bounty.status = 2; // Released
        self.bounties.set(&task_id, bounty);
    }

    pub fn refund(&mut self, task_id: String) {
        let mut bounty = self.get_bounty(&task_id);
        let caller = self.env().caller();

        if caller != bounty.verifier && caller != bounty.creator {
            self.env().revert(Error::NotCreatorOrVerifier);
        }

        if bounty.status == 2 || bounty.status == 3 {
            self.env().revert(Error::InvalidStatus);
        }

        self.env().transfer_tokens(&bounty.creator, &bounty.amount);

        bounty.status = 3; // Refunded
        self.bounties.set(&task_id, bounty);
    }

    pub fn get_bounty_status(&self, task_id: String) -> u8 {
        self.get_bounty(&task_id).status
    }

    // --- Private Helpers ---

    fn get_bounty(&self, task_id: &String) -> Bounty {
        match self.bounties.get(task_id) {
            Some(b) => b,
            None => self.env().revert(Error::BountyNotFound),
        }
    }
}
