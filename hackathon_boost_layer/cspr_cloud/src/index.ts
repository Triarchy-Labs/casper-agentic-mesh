import { CSPRCloudRestClient } from "./client.js";
import { CSPRCloudStreamClient } from "./stream.js";
import { EventsEndpoint } from "./endpoints/events.js";
import { CSPRCloudConfig } from "./types.js";
export * from "./types.js";

export class TriarchyCSPRCloud {
    private restClient: CSPRCloudRestClient;
    private streamClient: CSPRCloudStreamClient;
    public events: EventsEndpoint;

    constructor(config: CSPRCloudConfig) {
        this.restClient = new CSPRCloudRestClient(config);
        this.streamClient = new CSPRCloudStreamClient(config);
        this.events = new EventsEndpoint(this.restClient, this.streamClient);
    }
}
