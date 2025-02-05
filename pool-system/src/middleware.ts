import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import WebSocketSingleton from './lib/ws-singleton';

let initialized = false;

export async function middleware(request: NextRequest) {
    // Initialize WebSocket server only once
    if (!initialized) {
        const WS_PORT = parseInt(process.env.WS_PORT || '3001');
        try {
            WebSocketSingleton.getInstance(WS_PORT);
            initialized = true;
            console.log('WebSocket server initialized via middleware');
        } catch (error) {
            console.error('Failed to initialize WebSocket server:', error);
        }
    }
    return NextResponse.next();
}

// Configure which paths the middleware runs on
export const config = {
    matcher: [
        /*
         * Match all request paths except:
         * - _next/static (static files)
         * - _next/image (image optimization files)
         * - favicon.ico (favicon file)
         * - public folder
         */
        '/((?!_next/static|_next/image|favicon.ico|public).*)',
    ],
};
