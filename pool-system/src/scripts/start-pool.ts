import { spawn } from 'child_process';
import { PoolWebSocketServer } from '../lib/websocket-server';
import dotenv from 'dotenv';

dotenv.config();

const WS_PORT = parseInt(process.env.WS_PORT || '3001');

async function main() {
    // Start Next.js app
    const nextApp = spawn('npm', ['run', 'dev'], {
        stdio: 'inherit',
        shell: true
    });

    // Start WebSocket server
    console.log(`Starting WebSocket server on port ${WS_PORT}...`);
    try {
        new PoolWebSocketServer(WS_PORT);
        console.log('WebSocket server is running');
        console.log('Waiting for miners to connect...');
    } catch (error) {
        console.error('Failed to start WebSocket server:', error);
        nextApp.kill();
        process.exit(1);
    }

    // Handle process termination
    process.on('SIGINT', () => {
        console.log('\nShutting down...');
        nextApp.kill();
        process.exit(0);
    });
}

main();
