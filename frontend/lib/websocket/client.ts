export type MessageHandler = (message: Record<string, unknown>) => void;

export class WebSocketClient {
    private ws: WebSocket | null = null;
    private url: string;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 5;
    private reconnectDelay = 3000;
    private messageHandlers: Map<string, Set<MessageHandler>> = new Map();
    private connectionPromise: Promise<void> | null = null;

    constructor(url: string) {
        this.url = url;
    }

    async connect(): Promise<void> {
        if (this.connectionPromise) {
            return this.connectionPromise;
        }

        this.connectionPromise = new Promise((resolve, reject) => {
            try {
                this.ws = new WebSocket(this.url);

                this.ws.onopen = () => {
                    console.log('WebSocket connected');
                    this.reconnectAttempts = 0;
                    resolve();
                };

                this.ws.onmessage = (event) => {
                    try {
                        const message = JSON.parse(event.data);
                        this.handleMessage(message);
                    } catch (error) {
                        console.error('Failed to parse WebSocket message:', error);
                    }
                };

                this.ws.onerror = (error) => {
                    console.error('WebSocket error:', error);
                    reject(error);
                };

                this.ws.onclose = () => {
                    console.log('WebSocket disconnected');
                    this.connectionPromise = null;
                    this.attemptReconnect();
                };
            } catch (error) {
                reject(error);
            }
        });

        return this.connectionPromise;
    }

    private attemptReconnect(): void {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
            setTimeout(() => {
                this.connectionPromise = null;
                this.connect().catch(console.error);
            }, this.reconnectDelay);
        }
    }

    private handleMessage(message: Record<string, unknown>): void {
        const { type, channel, data } = message;

        if (type === 'event' && typeof channel === 'string') {
            const handlers = this.messageHandlers.get(channel);
            if (handlers) {
                handlers.forEach((handler) => handler(data as Record<string, unknown>));
            }
        }
    }

    subscribe(channel: string, handler: MessageHandler): void {
        if (!this.messageHandlers.has(channel)) {
            this.messageHandlers.set(channel, new Set());
        }
        this.messageHandlers.get(channel)!.add(handler);

        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.send({
                type: 'subscribe',
                channel,
            });
        }
    }

    unsubscribe(channel: string, handler?: MessageHandler): void {
        if (handler) {
            this.messageHandlers.get(channel)?.delete(handler);
        } else {
            this.messageHandlers.delete(channel);
        }

        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.send({
                type: 'unsubscribe',
                channel,
            });
        }
    }

    send(message: Record<string, unknown>): void {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        } else {
            console.warn('WebSocket is not connected');
        }
    }

    disconnect(): void {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        this.connectionPromise = null;
        this.messageHandlers.clear();
    }

    isConnected(): boolean {
        return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
    }
}
