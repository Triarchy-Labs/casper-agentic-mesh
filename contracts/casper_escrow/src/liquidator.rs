use odra::prelude::*;
use odra::casper_types::U512;
use odra::ContractRef;

#[odra::external_contract]
pub trait EscrowCross {
    fn submit_proof(&mut self, task_id: String, signature: String);
    fn release(&mut self, task_id: String) -> U512;
    fn submit_and_release(&mut self, task_id: String, signature: String) -> U512;
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

#[odra::event]
pub struct ArbitrageExecuted {
    pub target_task_id: String,
    pub profit_realized: U512,
    pub timestamp: u64,
}

#[odra::module(events = [ArbitrageExecuted])]
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
        if expected_bounty <= x402_cost {
            self.env().revert(Error::NotProfitable);
        }

        let profit = expected_bounty - x402_cost;

        // Perform single Cross-Contract Call to Escrow contract
        let mut escrow = EscrowCrossContractRef::new(self.env(), escrow_address);
        
        // Single XCC entry point: submits proof and releases escrowed funds in one call
        let released_amount = escrow.submit_and_release(target_task_id.clone(), proof_signature);
        
        let actual_profit = if released_amount > x402_cost {
            released_amount - x402_cost
        } else {
            profit
        };

        let now = self.env().get_block_time();
        let record = LiquidationRecord {
            target_task_id: target_task_id.clone(),
            profit_realized: actual_profit,
            timestamp: now,
        };

        self.liquidations.set(&target_task_id, record);
        
        let current_profit = self.total_profit.get_or_default();
        self.total_profit.set(current_profit.saturating_add(actual_profit));

        self.env().emit_event(ArbitrageExecuted {
            target_task_id,
            profit_realized: actual_profit,
            timestamp: now,
        });
    }

    pub fn get_total_profit(&self) -> U512 {
        self.total_profit.get_or_default()
    }
}
