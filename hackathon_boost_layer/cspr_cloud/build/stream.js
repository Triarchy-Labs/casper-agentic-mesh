import WebSocket from "ws";
import { EventEmitter } from "events";
export class CSPRCloudStreamClient extends EventEmitter {
    ws = null;
    apiKey;
    wssUrl;
    reconnectAttempts = 0;
    maxReconnectAttempts = Infinity;
    pingInterval = null;
    reconnectTimeout = null;
    isDisconnecting = false;
    pongReceived = true;
    lastActivity = Date.now();
    constructor(config) {
        super();
        this.apiKey = config.apiKey;
        const network = config.network || "testnet";
        let url = `wss://streaming.${network}.cspr.cloud/contract-events?api_key=${this.apiKey}`;
        if (config.contractPackageHash) {
            url += `&contract_package_hash=${config.contractPackageHash}`;
        }
        this.wssUrl = url;
        this.on('error', () => { });
    }
    connect() {
        this.isDisconnecting = false;
        if (this.ws) {
            if (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING) {
                return;
            }
            this.ws.removeAllListeners();
        }
        this.emit("log", "Connecting to CSPR.cloud WebSocket...");
        const socket = new WebSocket(this.wssUrl);
        this.ws = socket;
        socket.on("open", () => {
            if (this.ws !== socket)
                return;
            this.emit("log", "CSPR.cloud WebSocket Connected");
            this.reconnectAttempts = 0;
            this.emit("connected");
            this.pongReceived = true;
            this.lastActivity = Date.now();
            this.pingInterval = setInterval(() => {
                if (socket.readyState === WebSocket.OPEN) {
                    if (!this.pongReceived) {
                        this.emit("error", new Error("WebSocket dead (no pong). Terminating."));
                        socket.terminate();
                        return;
                    }
                    if (Date.now() - this.lastActivity > 60000) {
                        this.emit("error", new Error("Zombie socket: TCP alive but no application data > 60s."));
                        socket.terminate();
                        return;
                    }
                    this.pongReceived = false;
                    socket.ping();
                }
            }, 30000);
        });
        socket.on("pong", () => {
            this.pongReceived = true;
        });
        socket.on("message", (data) => {
            if (this.ws !== socket)
                return;
            this.lastActivity = Date.now();
            try {
                // Avoid calling data.toString() directly on massive buffers to prevent event loop blocking
                let textData;
                if (Buffer.isBuffer(data)) {
                    // Small buffers can be converted safely, but large ones should be handled carefully
                    textData = data.toString('utf-8');
                }
                else if (data instanceof ArrayBuffer) {
                    textData = Buffer.from(data).toString('utf-8');
                }
                else {
                    textData = Buffer.concat(data).toString('utf-8');
                }
                const message = JSON.parse(textData);
                if (message.type === "ContractEvent" && message.payload) {
                    this.emit("event", message.payload);
                }
                else if (message.type === "Heartbeat") {
                    this.emit("heartbeat", message.timestamp);
                }
            }
            catch (e) {
                this.emit("error", new Error(`Parse error: ${e.message}`));
            }
        });
        socket.on("close", () => {
            if (this.ws !== socket)
                return;
            this.emit("log", "CSPR.cloud WebSocket Disconnected");
            this.emit("disconnected");
            this.cleanup();
            if (!this.isDisconnecting) {
                this.reconnect();
            }
            // Now safe to nullify after events are emitted
            if (this.isDisconnecting) {
                this.ws = null;
            }
        });
        socket.on("error", (error) => {
            if (this.ws !== socket)
                return;
            this.emit("error", new Error(`WebSocket Error: ${error.message}`));
        });
    }
    disconnect() {
        this.isDisconnecting = true;
        this.cleanup();
        if (this.ws) {
            this.ws.close();
            // Do NOT nullify this.ws here. Let the "close" event handler do it so "disconnected" fires.
        }
    }
    cleanup() {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
            this.pingInterval = null;
        }
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }
    }
    reconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            this.emit("error", new Error("Max reconnect attempts reached."));
            return;
        }
        this.reconnectAttempts++;
        const backoff = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 60000);
        this.emit("log", `Reconnecting in ${backoff}ms (Attempt ${this.reconnectAttempts})...`);
        this.reconnectTimeout = setTimeout(() => this.connect(), backoff);
    }
}
