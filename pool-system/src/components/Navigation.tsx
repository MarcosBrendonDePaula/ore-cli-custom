'use client';

import Link from 'next/link';
import { useWebSocket } from './WebSocketProvider';

export default function Navigation() {
  const { isConnected } = useWebSocket();
  return (
    <nav className="bg-gray-800 text-white p-4">
      <div className="container mx-auto flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Link href="/" className="text-xl font-bold">
            Mining Pool
          </Link>
          <Link href="/hashes" className="hover:text-gray-300">
            Hashes
          </Link>
          <Link href="/settings" className="hover:text-gray-300">
            Settings
          </Link>
        </div>
        <div className="flex items-center space-x-4">
          <div className="text-sm">
            WebSocket Status: <span className={isConnected ? "text-green-400" : "text-red-400"}>
              {isConnected ? "Connected" : "Disconnected"}
            </span>
          </div>
        </div>
      </div>
    </nav>
  );
}
