import { CSPRCloudRestClient } from "../client.js";
import { CSPRCloudStreamClient } from "../stream.js";
import { ContractEvent } from "../types.js";
import { EventEmitter } from "events";

export class EventsEndpoint extends EventEmitter {
    private rest: CSPRCloudRestClient;
    private stream: CSPRCloudStreamClient;

    // Bound handlers for easy removal
    private onEvent = (e: ContractEvent) => this.emit("event", e);
    private onError = (err: any) => this.emit("error", err);
    private onConnected = () => this.emit("connected");
    private onDisconnected = () => this.emit("disconnected");

    constructor(restClient: CSPRCloudRestClient, streamClient: CSPRCloudStreamClient) {
        super();
        this.rest = restClient;
        this.stream = streamClient;

        // Forward stream events
        this.stream.on("event", this.onEvent);
        this.stream.on("error", this.onError);
        this.stream.on("connected", this.onConnected);
        this.stream.on("disconnected", this.onDisconnected);
    }

    /**
     * Gets historical events via REST
     */
    public async getHistory(contractPackageHash: string, limit: number = 10, cursor?: string) {
        return this.rest.getEvents(contractPackageHash, limit, cursor);
    }

    /**
     * Starts listening for real-time events via WebSocket
     */
    public listen() {
        this.stream.connect();
    }

    /**
     * Stops the real-time stream
     */
    public stopListening() {
        this.stream.disconnect();
    }
    
    /**
     * Cleans up listeners to prevent memory leaks if instantiated multiple times
     */
    public destroy() {
        this.stream.off("event", this.onEvent);
        this.stream.off("error", this.onError);
        this.stream.off("connected", this.onConnected);
        this.stream.off("disconnected", this.onDisconnected);
        this.removeAllListeners();
    }
}
