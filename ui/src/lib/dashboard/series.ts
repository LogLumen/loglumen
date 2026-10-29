import type { Bucket } from '#lib/types/api.ts';

export interface RunSeries {
	run: string;
	buckets: Bucket[];
}

const PALETTE_REST = ['#0F172A', '#C2410C', '#0E7490', '#7C3AED', '#BE185D', '#A16207', '#64748B'];

export const runColor = (index: number) =>
	index === 0 ? '#1D4ED8' : PALETTE_REST[(index - 1) % PALETTE_REST.length];

/** The bucket nearest to `x`, or nothing when `x` lies more than half a bucket outside the series. */
export function bucketAt(buckets: Bucket[], x: number): Bucket | undefined {
	if (!buckets.length) return undefined;
	const first = buckets[0].x;
	const last = buckets[buckets.length - 1].x;
	const slack = buckets.length > 1 ? (last - first) / (buckets.length - 1) / 2 : 0;
	if (x < first - slack || x > last + slack) return undefined;
	return buckets.reduce((best, b) => (Math.abs(b.x - x) < Math.abs(best.x - x) ? b : best));
}

export function formatValue(v: number): string {
	if (v === 0) return '0';
	const abs = Math.abs(v);
	if (abs < 1e-3 || abs >= 1e6) return v.toExponential(2);
	if (abs >= 1000) return v.toLocaleString('en-US', { maximumFractionDigits: 1 });
	return v.toFixed(4);
}

const clock = new Intl.DateTimeFormat('en-GB', {
	hour: '2-digit',
	minute: '2-digit',
	second: '2-digit'
});

export function formatX(axis: string, v: number): string {
	if (axis === 'step') return `step ${Math.max(0, Math.round(v)).toLocaleString('en-US')}`;
	if (axis === 'timestamp') return clock.format(v);
	return v.toLocaleString('en-US', { maximumFractionDigits: 2 });
}
