import { TriarchyMcpServer } from "./server.js";
const server = new TriarchyMcpServer();
server.run().catch(console.error);
