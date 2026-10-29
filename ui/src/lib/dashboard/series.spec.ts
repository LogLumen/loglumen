import { describe, expect, it } from 'vitest';
import { bucketAt, formatValue, formatX, runColor } from './series.ts';

const bucket = (x: number) => ({ x, avg: x, min: x, max: x, count: 1 });

describe('bucketAt', () => {
	const buckets = [0, 10, 20].map(bucket);

	it('picks the nearest bucket', () => {
		expect(bucketAt(buckets, 13)?.x).toBe(10);
		expect(bucketAt(buckets, 16)?.x).toBe(20);
	});

	it('allows half a bucket of slack past either end', () => {
		expect(bucketAt(buckets, -5)?.x).toBe(0);
		expect(bucketAt(buckets, 25)?.x).toBe(20);
		expect(bucketAt(buckets, 26)).toBeUndefined();
	});

	it('handles empty and single-bucket series', () => {
		expect(bucketAt([], 0)).toBeUndefined();
		expect(bucketAt([bucket(5)], 5)?.x).toBe(5);
		expect(bucketAt([bucket(5)], 6)).toBeUndefined();
	});
});

describe('formatting', () => {
	it('formats values by magnitude', () => {
		expect(formatValue(0.123456)).toBe('0.1235');
		expect(formatValue(0.0003)).toBe('3.00e-4');
		expect(formatValue(12345.67)).toBe('12,345.7');
	});

	it('formats step axes', () => {
		expect(formatX('step', 19296.4)).toBe('step 19,296');
	});

	it('uses the accent for the first run', () => {
		expect(runColor(0)).toBe('#1D4ED8');
		expect(runColor(1)).toBe('#0F172A');
	});
});
