'use client';

import { createContext, useContext, useEffect, useState } from 'react';

type WebSocketContextType = {
  isConnected: boolean;
};

const WebSocketContext = createContext<WebSocketContextType>({
  isConnected: false,
});

export function useWebSocket() {
  return useContext(WebSocketContext);
}

export function WebSocketProvider({ children }: { children: React.ReactNode }) {
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const checkWebSocketServer = async () => {
      try {
        const response = await fetch('/api/ws');
        if (response.ok) {
          setIsConnected(true);
        }
      } catch (error) {
        console.error('WebSocket server check failed:', error);
        setIsConnected(false);
      }
    };

    checkWebSocketServer();
    const interval = setInterval(checkWebSocketServer, 5000);

    return () => clearInterval(interval);
  }, []);

  return (
    <WebSocketContext.Provider value={{ isConnected }}>
      {children}
    </WebSocketContext.Provider>
  );
}
