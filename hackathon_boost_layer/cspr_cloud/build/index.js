import { CSPRCloudRestClient } from "./client.js";
import { CSPRCloudStreamClient } from "./stream.js";
import { EventsEndpoint } from "./endpoints/events.js";
export * from "./types.js";
export class TriarchyCSPRCloud {
    restClient;
    streamClient;
    events;
    constructor(config) {
        this.restClient = new CSPRCloudRestClient(config);
        this.streamClient = new CSPRCloudStreamClient(config);
        this.events = new EventsEndpoint(this.restClient, this.streamClient);
    }
}
