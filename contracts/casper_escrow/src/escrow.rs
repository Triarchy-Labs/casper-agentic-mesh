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
    AlreadyExists = 6,
    NotHunter = 7,
    InvalidSignature = 8,
}

#[odra::event]
pub struct Deposited {
    pub task_id: String,
    pub creator: Address,
    pub hunter: Address,
    pub verifier: Address,
    pub amount: U512,
}

#[odra::event]
pub struct ProofSubmitted {
    pub task_id: String,
    pub hunter: Address,
    pub signature: String,
}

#[odra::event]
pub struct Released {
    pub task_id: String,
    pub hunter: Address,
    pub amount: U512,
}

#[odra::event]
pub struct Refunded {
    pub task_id: String,
    pub creator: Address,
    pub amount: U512,
}

#[odra::module(events = [Deposited, ProofSubmitted, Released, Refunded])]
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

        if self.bounties.get(&task_id).is_some() {
            self.env().revert(Error::AlreadyExists);
        }

        let creator = self.env().caller();

        let bounty = Bounty {
            creator,
            hunter,
            verifier,
            amount,
            status: 0,
        };

        self.bounties.set(&task_id, bounty);

        self.env().emit_event(Deposited {
            task_id,
            creator,
            hunter,
            verifier,
            amount,
        });
    }

    pub fn submit_proof(&mut self, task_id: String, signature: String) {
        let mut bounty = self.get_bounty(&task_id);

        if self.env().caller() != bounty.hunter {
            self.env().revert(Error::NotHunter);
        }

        if signature.is_empty() {
            self.env().revert(Error::InvalidSignature);
        }

        if bounty.status != 0 {
            self.env().revert(Error::InvalidStatus);
        }

        bounty.status = 1;
        self.bounties.set(&task_id, bounty);

        self.env().emit_event(ProofSubmitted {
            task_id,
            hunter: self.env().caller(),
            signature,
        });
    }

    pub fn release(&mut self, task_id: String) -> U512 {
        let mut bounty = self.get_bounty(&task_id);
        let caller = self.env().caller();

        if caller != bounty.verifier {
            self.env().revert(Error::NotVerifier);
        }

        if bounty.status != 1 {
            self.env().revert(Error::InvalidStatus);
        }

        let hunter = bounty.hunter;
        let amount = bounty.amount;

        self.env().transfer_tokens(&hunter, &amount);

        bounty.status = 2; // Released
        self.bounties.set(&task_id, bounty);

        self.env().emit_event(Released {
            task_id,
            hunter,
            amount,
        });

        amount
    }

    pub fn submit_and_release(&mut self, task_id: String, signature: String) -> U512 {
        self.submit_proof(task_id.clone(), signature);
        self.release(task_id)
    }

    pub fn refund(&mut self, task_id: String) {
        let mut bounty = self.get_bounty(&task_id);
        let caller = self.env().caller();

        if caller != bounty.verifier && caller != bounty.creator {
            self.env().revert(Error::NotCreatorOrVerifier);
        }

        if bounty.status == 1 && caller != bounty.verifier {
            self.env().revert(Error::InvalidStatus);
        }

        if bounty.status == 2 || bounty.status == 3 {
            self.env().revert(Error::InvalidStatus);
        }

        let creator = bounty.creator;
        let amount = bounty.amount;

        self.env().transfer_tokens(&creator, &amount);

        bounty.status = 3; // Refunded
        self.bounties.set(&task_id, bounty);

        self.env().emit_event(Refunded {
            task_id,
            creator,
            amount,
        });
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
