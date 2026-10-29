import { describe, expect, it } from 'vitest';
import { clampLayout, collides, compact, nextFreeSlot, resolve, SEED_CHARTS } from './layout.ts';

const at = (id: string, x: number, y: number, w: number, h: number) => ({
	id,
	layout: { x, y, w, h }
});
const byId = (items: ReturnType<typeof at>[]) =>
	Object.fromEntries(items.map((i) => [i.id, [i.layout.x, i.layout.y, i.layout.w, i.layout.h]]));

describe('collides', () => {
	it('detects overlap but not edge contact', () => {
		expect(collides({ x: 0, y: 0, w: 2, h: 1 }, { x: 1, y: 0, w: 2, h: 1 })).toBe(true);
		expect(collides({ x: 0, y: 0, w: 2, h: 1 }, { x: 2, y: 0, w: 2, h: 1 })).toBe(false);
		expect(collides({ x: 0, y: 0, w: 2, h: 1 }, { x: 0, y: 1, w: 2, h: 1 })).toBe(false);
	});
});

describe('clampLayout', () => {
	it('keeps the layout inside the 6 columns', () => {
		expect(clampLayout({ x: 5.4, y: -2, w: 3, h: 0 })).toEqual({ x: 3, y: 0, w: 3, h: 1 });
		expect(clampLayout({ x: 0, y: 0, w: 9, h: 2 })).toEqual({ x: 0, y: 0, w: 6, h: 2 });
	});
});

describe('resolve', () => {
	it('pushes overlapped charts down', () => {
		const out = resolve([at('a', 0, 0, 2, 1), at('b', 2, 0, 2, 1)], 'a', {
			x: 2,
			y: 0,
			w: 2,
			h: 1
		});
		expect(byId(out)).toEqual({ a: [2, 0, 2, 1], b: [2, 1, 2, 1] });
	});

	it('floats charts up into freed space', () => {
		const out = resolve([at('a', 0, 0, 6, 1), at('b', 0, 1, 2, 1)], 'a', {
			x: 0,
			y: 3,
			w: 6,
			h: 1
		});
		expect(byId(out)).toEqual({ a: [0, 1, 6, 1], b: [0, 0, 2, 1] });
	});

	it('grows a resized chart over its neighbours', () => {
		const out = resolve([at('a', 0, 0, 2, 1), at('b', 2, 0, 2, 1)], 'a', {
			x: 0,
			y: 0,
			w: 4,
			h: 1
		});
		expect(byId(out)).toEqual({ a: [0, 0, 4, 1], b: [2, 1, 2, 1] });
	});
});

describe('compact', () => {
	it('closes vertical gaps', () => {
		expect(byId(compact([at('a', 0, 3, 2, 1), at('b', 0, 6, 2, 2)]))).toEqual({
			a: [0, 0, 2, 1],
			b: [0, 1, 2, 2]
		});
	});
});

describe('nextFreeSlot', () => {
	it('finds the gap after the seeded charts', () => {
		expect(
			nextFreeSlot(
				SEED_CHARTS.map((c) => c.layout),
				2,
				1
			)
		).toEqual({ x: 4, y: 3, w: 2, h: 1 });
	});
});
