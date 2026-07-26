export interface CSPRCloudConfig {
    apiKey: string;
    network?: "mainnet" | "testnet";
    contractPackageHash?: string;
}

export interface ContractEvent {
    id: string;
    contract_package_hash: string;
    event_type: string;
    data: any;
    timestamp: string;
}

export interface StreamEventFrame {
    type: "ContractEvent" | "Heartbeat" | "Error";
    payload?: ContractEvent;
    timestamp: string;
    error?: string;
}
