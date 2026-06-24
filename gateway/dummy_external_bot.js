/* eslint-disable */
// Dummy OpenClaw External Fast-API Bot for Hackathon Video Recording
// Runs on a separate port (e.g., 3001) to simulate external network.

const express = require('express');
const cors = require('cors');
const app = express();
app.use(cors());
app.use(express.json());

const PORT = 3001;

// Simulate OpenClaw external API
app.post('/api/hire', (req, res) => {
    const { task } = req.body;
    
    console.log(`\n[EXTERNAL OPENCLAW BOT] Received Task Over the Mesh!`);
    console.log(`[EXTERNAL OPENCLAW BOT] Verifying 50% USDC transferred via Soroban... OK.`);
    console.log(`[EXTERNAL OPENCLAW BOT] Task Details: "${task}"`);
    console.log(`[EXTERNAL OPENCLAW BOT] Executing local LLM request (mocked)...`);

    // Simulate thinking delay (2 seconds) for realistic video recording
    setTimeout(() => {
        let result = "";
        if (task.toLowerCase().includes("vulnerabilities") || task.toLowerCase().includes("audit")) {
            result = "No re-entrancy bugs detected in the provided Soroban contract.";
        } else {
            result = "Task executed successfully by External Agent 0x4B3. Ready for next block.";
        }

        res.json({
            agent_id: "openclaw_dummy_bot_0x4b3",
            status: "success",
            result: result,
            usdc_received: "Yes (50% cut of the original)"
        });
        
        console.log(`[EXTERNAL OPENCLAW BOT] Response sent back to P2P Router.`);
    }, 2000);
});

app.listen(PORT, () => {
    console.log(`[EXTERNAL OPENCLAW BOT] Alive and listening for x402 tasks on port ${PORT}...`);
    console.log(`Run this in a parallel terminal during the Hacker video recording!`);
});
