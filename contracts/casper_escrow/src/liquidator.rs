use odra::prelude::*;
use odra::casper_types::U512;
use odra::ContractRef;

#[odra::external_contract]
pub trait EscrowCross {
    fn submit_proof(&mut self, task_id: String, signature: String);
    fn release(&mut self, task_id: String);
}

#[odra::odra_type]
pub struct LiquidationRecord {
    pub target_task_id: String,
    pub profit_realized: U512,
    pub timestamp: u64,
}

#[odra::odra_error]
pub enum Error {
    NotProfitable = 1,
    Unauthorized = 2,
    FlashLoanFailed = 3,
}

#[odra::module]
pub struct X402Liquidator {
    pub owner: Var<Address>,
    pub liquidations: Mapping<String, LiquidationRecord>,
    pub total_profit: Var<U512>,
}

#[odra::module]
impl X402Liquidator {
    #[odra(init)]
    pub fn init(&mut self) {
        self.owner.set(self.env().caller());
        self.total_profit.set(U512::zero());
    }

    /// Executes cognitive arbitrage by comparing the cost to acquire a task's solution
    /// versus the reward escrowed for that task.
    pub fn execute_arbitrage(
        &mut self,
        target_task_id: String,
        escrow_address: Address,
        x402_cost: U512,
        expected_bounty: U512,
        proof_signature: String,
    ) {
        let caller = self.env().caller();
        let owner = self.owner.get().unwrap_or_else(|| self.env().revert(Error::Unauthorized));
        if caller != owner {
            self.env().revert(Error::Unauthorized);
        }

        // Must be strictly profitable to execute (profit > x402 cost + gas buffer)
        // In a real environment, gas is paid in CSPR, so we ensure a net-positive yield.
        if expected_bounty <= x402_cost {
            self.env().revert(Error::NotProfitable);
        }

        let profit = expected_bounty - x402_cost;

        // Perform the actual Cross-Contract Call to the Escrow contract
        let mut escrow = EscrowCrossContractRef::new(self.env(), escrow_address);
        
        // 1. Submit the generated cognitive proof
        escrow.submit_proof(target_task_id.clone(), proof_signature);
        
        // 2. Trigger release (Assuming X402Liquidator acts as or coordinates with the Verifier)
        escrow.release(target_task_id.clone());

        let record = LiquidationRecord {
            target_task_id: target_task_id.clone(),
            profit_realized: profit,
            timestamp: self.env().get_block_time(),
        };

        self.liquidations.set(&target_task_id, record);
        
        let current_profit = self.total_profit.get_or_default();
        self.total_profit.set(current_profit + profit);
    }

    pub fn get_total_profit(&self) -> U512 {
        self.total_profit.get_or_default()
    }
}
