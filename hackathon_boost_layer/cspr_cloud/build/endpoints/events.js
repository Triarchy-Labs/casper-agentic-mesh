import { EventEmitter } from "events";
export class EventsEndpoint extends EventEmitter {
    rest;
    stream;
    // Bound handlers for easy removal
    onEvent = (e) => this.emit("event", e);
    onError = (err) => this.emit("error", err);
    onConnected = () => this.emit("connected");
    onDisconnected = () => this.emit("disconnected");
    constructor(restClient, streamClient) {
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
    async getHistory(contractPackageHash, limit = 10, cursor) {
        return this.rest.getEvents(contractPackageHash, limit, cursor);
    }
    /**
     * Starts listening for real-time events via WebSocket
     */
    listen() {
        this.stream.connect();
    }
    /**
     * Stops the real-time stream
     */
    stopListening() {
        this.stream.disconnect();
    }
    /**
     * Cleans up listeners to prevent memory leaks if instantiated multiple times
     */
    destroy() {
        this.stream.off("event", this.onEvent);
        this.stream.off("error", this.onError);
        this.stream.off("connected", this.onConnected);
        this.stream.off("disconnected", this.onDisconnected);
        this.removeAllListeners();
    }
}
