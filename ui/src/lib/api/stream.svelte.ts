import type { Bucket, IncomingMessage, Query } from '#lib/types/api.ts';

export type StreamStatus = 'connecting' | 'connected' | 'reconnecting' | 'disconnected';

interface Pending {
	query: Query;
	resolve: (buckets: Bucket[]) => void;
	reject: (error: Error) => void;
}

const MAX_BACKOFF_MS = 10_000;
const DISCONNECTED_AFTER = 4;

/** One shared `/api/charts/ws` connection; requests in flight are re-sent after a reconnect. */
class ChartStream {
	status = $state<StreamStatus>('connecting');

	#socket: WebSocket | null = null;
	#pending = new Map<string, Pending>();
	#attempts = 0;
	#seq = 0;

	query(query: Query): Promise<Bucket[]> {
		const id = `q${++this.#seq}`;
		return new Promise((resolve, reject) => {
			this.#pending.set(id, { query, resolve, reject });
			if (this.#socket?.readyState === WebSocket.OPEN) this.#send(id, query);
			else this.connect();
		});
	}

	connect() {
		if (this.#socket) return;
		const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
		const socket = new WebSocket(`${protocol}//${location.host}/api/charts/ws`);
		this.#socket = socket;

		socket.onopen = () => {
			this.#attempts = 0;
			this.status = 'connected';
			for (const [id, { query }] of this.#pending) this.#send(id, query);
		};
		socket.onmessage = (event) => this.#receive(JSON.parse(event.data) as IncomingMessage);
		socket.onclose = () => {
			this.#socket = null;
			this.#attempts++;
			this.status = this.#attempts >= DISCONNECTED_AFTER ? 'disconnected' : 'reconnecting';
			const delay = Math.min(MAX_BACKOFF_MS, 500 * 2 ** this.#attempts);
			setTimeout(() => this.connect(), delay);
		};
	}

	#send(id: string, query: Query) {
		this.#socket?.send(JSON.stringify({ action: 'subscribe', id, query }));
	}

	#receive(message: IncomingMessage) {
		const pending = message.id === null ? undefined : this.#pending.get(message.id);
		if (!pending || message.id === null) {
			if (message.action === 'error') console.error('chart stream:', message.message);
			return;
		}
		this.#pending.delete(message.id);
		if (message.action === 'replace_all') pending.resolve(message.buckets);
		else pending.reject(new Error(message.message));
	}
}

export const stream = new ChartStream();
