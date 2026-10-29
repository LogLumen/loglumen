// Same-origin `/api`: proxied to axum by Vite in dev, served by axum in prod.
const BASE = '/api';

export class ApiError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
	}
}

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE}${path}`, {
		...init,
		headers: { 'content-type': 'application/json', ...init?.headers }
	});
	if (!res.ok) throw new ApiError(res.status, await res.text());
	return (await res.json()) as T;
}
