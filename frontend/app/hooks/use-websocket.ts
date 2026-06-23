import { useEffect, useState } from 'react';

interface WsState<T> {
  data: T | null;
  lastUpdated: Date | null;
}

export const useWebSocketData = <T>(url: string): WsState<T> => {
  const [state, setState] = useState<WsState<T>>({
    data: null,
    lastUpdated: null,
  });

  useEffect(() => {
    let ws: WebSocket;
    let destroyed = false;
    let retryTimer: ReturnType<typeof setTimeout>;

    const handleMessage = (e: MessageEvent) => {
      try {
        setState({ data: JSON.parse(e.data) as T, lastUpdated: new Date() });
      } catch {
        // ignore malformed messages
      }
    };

    const handleClose = () => {
      if (!destroyed) {
        retryTimer = setTimeout(connect, 2000);
      }
    };

    const handleError = () => {
      ws.close();
    };

    let connect: () => void;
    connect = () => {
      ws = new WebSocket(url);
      ws.addEventListener('message', handleMessage);
      ws.addEventListener('close', handleClose);
      ws.addEventListener('error', handleError);
    };

    connect();

    return () => {
      destroyed = true;
      clearTimeout(retryTimer);
      ws.removeEventListener('close', handleClose);
      ws.close();
    };
  }, [url]);

  return state;
};
