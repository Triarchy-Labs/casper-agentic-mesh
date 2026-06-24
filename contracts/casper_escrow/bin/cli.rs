//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use casper_escrow::EscrowContract;
use odra::host::{HostEnv, NoArgs};
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer, DeployerExt,
    OdraCli, 
};

/// Deploys the `EscrowContract` and adds it to the container.
pub struct EscrowDeployScript;

impl DeployScript for EscrowDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer
    ) -> Result<(), odra_cli::deploy::Error> {
        let _escrow = EscrowContract::load_or_deploy(
            &env,
            NoArgs,
            container,
            350_000_000_000 // Adjust gas limit as needed
        )?;

        Ok(())
    }
}

/// Scenario that deposits into the deployed `EscrowContract`.
pub struct DepositScenario;

impl Scenario for DepositScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("task_id", "The ID of the task", NamedCLType::String),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args
    ) -> Result<(), Error> {
        let _contract = container.contract_ref::<EscrowContract>(env)?;
        // Scenario actions can be added here
        Ok(())
    }
}

impl ScenarioMetadata for DepositScenario {
    const NAME: &'static str = "deposit";
    const DESCRIPTION: &'static str = "Deposits funds into the escrow contract";
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for casper_escrow smart contract")
        .deploy(EscrowDeployScript)
        .contract::<EscrowContract>()
        .scenario(DepositScenario)
        .build()
        .run();
}
