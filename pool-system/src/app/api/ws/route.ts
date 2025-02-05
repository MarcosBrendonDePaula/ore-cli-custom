import { NextResponse } from 'next/server';
import WebSocketSingleton from '../../../lib/ws-singleton';

export async function GET() {
    const WS_PORT = parseInt(process.env.WS_PORT || '3001');
    try {
        WebSocketSingleton.getInstance(WS_PORT);
        return NextResponse.json({ 
            status: 'WebSocket server is running'
        });
    } catch (error) {
        console.error('Failed to start WebSocket server:', error);
        return NextResponse.json({ 
            status: 'WebSocket server failed to start',
            error: error instanceof Error ? error.message : 'Unknown error'
        }, { status: 500 });
    }
}
