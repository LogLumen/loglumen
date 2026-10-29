// Mirrors be/src/charts.rs and be/src/query.rs until the BE publishes an OpenAPI spec.

export interface Project {
	name: string;
}

export interface Run {
	name: string;
}

export interface Layout {
	x: number;
	y: number;
	w: number;
	h: number;
}

export interface ChartConfig {
	title: string;
	key: string;
	select: string;
	/** `step`, `timestamp` or any other logged field */
	x: string;
	layout: Layout;
}

export interface Dashboard {
	id: string;
	project_name: string;
	config: Record<string, unknown>;
	updated_at: string;
}

export interface Chart {
	id: string;
	dashboard_id: string;
	config: ChartConfig;
	updated_at: string;
}

export type Field = 'timestamp' | { metric: string };

export type Filter =
	| { type: 'and' | 'or'; filters: Filter[] }
	| { type: 'cmp'; field: Field; op: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte'; value: number }
	| { type: 'in'; field: Field; values: number[] }
	| { type: 'run_in' | 'run_not_in'; runs: string[] };

export interface Query {
	project_name: string;
	key: string;
	x: Field;
	select: string;
	filter?: Filter;
	max_points: number;
}

export interface Bucket {
	x: number;
	avg: number;
	min: number;
	max: number;
	count: number;
}

export type IncomingMessage =
	| { action: 'replace_all'; id: string; buckets: Bucket[] }
	| { action: 'error'; id: string | null; message: string };
