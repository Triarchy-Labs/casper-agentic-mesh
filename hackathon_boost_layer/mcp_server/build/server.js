import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { CallToolRequestSchema, ListToolsRequestSchema } from "@modelcontextprotocol/sdk/types.js";
import { zodToJsonSchema } from "zod-to-json-schema";
import { tools } from "./tools/index.js";
// ==========================================
// TRUE ROOT STDIO GUARD: Prevent MCP protocol corruption
// ==========================================
// All stdout bound console methods must be routed to stderr natively.
// Calling the original stdout methods will corrupt JSON-RPC.
console.log = console.error;
console.info = console.error;
console.warn = console.error;
console.debug = console.error;
console.dir = console.error;
console.table = console.error;
// Prevent uncaught exceptions from writing naked stack traces to stdout
process.on('uncaughtException', (err) => {
    console.error("[Uncaught Exception]", err);
    process.exit(1);
});
process.on('unhandledRejection', (reason) => {
    console.error("[Unhandled Rejection]", reason);
});
// ==========================================
export class TriarchyMcpServer {
    server;
    constructor() {
        this.server = new Server({
            name: "casper-agentic-mesh",
            version: "1.0.0",
        }, {
            capabilities: {
                tools: {},
            },
        });
        this.setupToolHandlers();
        // Error handling
        this.server.onerror = (error) => console.error("[MCP Error]", error);
        // Graceful shutdown signals
        const shutdown = async () => {
            console.error("Shutting down MCP server...");
            await this.server.close();
            process.exit(0);
        };
        process.on('SIGINT', shutdown);
        process.on('SIGTERM', shutdown);
    }
    setupToolHandlers() {
        // LIST TOOLS
        this.server.setRequestHandler(ListToolsRequestSchema, async () => {
            return {
                tools: tools.map(t => ({
                    name: t.name,
                    description: t.description,
                    inputSchema: zodToJsonSchema(t.schema)
                }))
            };
        });
        // CALL TOOL
        this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
            const toolName = request.params.name;
            const toolArgs = request.params.arguments || {};
            const tool = tools.find(t => t.name === toolName);
            if (!tool) {
                throw new Error(`Tool not found: ${toolName}`);
            }
            try {
                // Parse arguments properly via zod
                const parsedArgs = tool.schema.parse(toolArgs);
                const result = await tool.handler(parsedArgs);
                return {
                    content: [
                        {
                            type: "text",
                            text: JSON.stringify(result, null, 2)
                        }
                    ]
                };
            }
            catch (error) {
                const message = error instanceof Error ? error.message : String(error);
                return {
                    content: [
                        {
                            type: "text",
                            text: `Tool Execution Failed: ${message}`
                        }
                    ],
                    isError: true
                };
            }
        });
    }
    async run() {
        const transport = new StdioServerTransport();
        await this.server.connect(transport);
        console.error("Triarchy Casper MCP Server running on stdio");
    }
}
