import type { ChartConfig, Layout } from '#lib/types/api.ts';

export const COLS = 6;
export const ROW_HEIGHT = 200;

export interface Placed {
	id: string;
	layout: Layout;
}

export const SEED_CHARTS: ChartConfig[] = [
	{ title: 'loss', key: 'train', select: 'loss', x: 'step', layout: { x: 0, y: 0, w: 6, h: 2 } },
	{
		title: 'val_loss',
		key: 'eval',
		select: 'val_loss',
		x: 'timestamp',
		layout: { x: 0, y: 2, w: 2, h: 1 }
	},
	{
		title: 'accuracy',
		key: 'eval',
		select: 'accuracy',
		x: 'timestamp',
		layout: { x: 2, y: 2, w: 2, h: 1 }
	},
	{
		title: 'learning_rate',
		key: 'train',
		select: 'learning_rate',
		x: 'step',
		layout: { x: 4, y: 2, w: 2, h: 1 }
	},
	{
		title: 'grad_norm',
		key: 'train',
		select: 'grad_norm',
		x: 'timestamp',
		layout: { x: 0, y: 3, w: 4, h: 1 }
	}
];

export const collides = (a: Layout, b: Layout) =>
	a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;

export function clampLayout({ x, y, w, h }: Layout): Layout {
	w = Math.min(COLS, Math.max(1, Math.round(w)));
	h = Math.max(1, Math.round(h));
	x = Math.min(COLS - w, Math.max(0, Math.round(x)));
	y = Math.max(0, Math.round(y));
	return { x, y, w, h };
}

const byPosition = (a: Placed, b: Placed) => a.layout.y - b.layout.y || a.layout.x - b.layout.x;

/** Puts `movedId` at `target`, pushes whatever it overlaps down, then lets everything float up. */
export function resolve(items: Placed[], movedId: string, target: Layout): Placed[] {
	const placed: Placed[] = [{ id: movedId, layout: clampLayout(target) }];
	for (const item of items.filter((i) => i.id !== movedId).sort(byPosition)) {
		const layout = { ...item.layout };
		while (placed.some((p) => collides(p.layout, layout))) layout.y++;
		placed.push({ id: item.id, layout });
	}
	return compact(placed, movedId);
}

export function compact(items: Placed[], priorityId?: string): Placed[] {
	const sorted = [...items].sort(
		(a, b) =>
			a.layout.y - b.layout.y ||
			Number(b.id === priorityId) - Number(a.id === priorityId) ||
			a.layout.x - b.layout.x
	);
	const out: Placed[] = [];
	for (const item of sorted) {
		const layout = { ...item.layout };
		while (layout.y > 0 && !out.some((o) => collides(o.layout, { ...layout, y: layout.y - 1 }))) {
			layout.y--;
		}
		out.push({ id: item.id, layout });
	}
	return out;
}

/** First `w`×`h` slot, scanning rows top to bottom, that overlaps nothing. */
export function nextFreeSlot(layouts: Layout[], w: number, h: number): Layout {
	for (let y = 0; ; y++) {
		for (let x = 0; x + w <= COLS; x++) {
			const slot = { x, y, w, h };
			if (!layouts.some((l) => collides(l, slot))) return slot;
		}
	}
}
