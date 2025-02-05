import { PoolWebSocketServer } from './websocket-server';

class WebSocketSingleton {
    private static instance: PoolWebSocketServer | null = null;

    static getInstance(port: number): PoolWebSocketServer {
        if (!WebSocketSingleton.instance) {
            WebSocketSingleton.instance = new PoolWebSocketServer(port);
            console.log(`WebSocket server initialized on port ${port}`);
        }
        return WebSocketSingleton.instance;
    }
}

export default WebSocketSingleton;
