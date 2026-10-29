<script lang="ts">
	import AddChartCard from './AddChartCard.svelte';
	import MetricChart from './MetricChart.svelte';
	import { COLS, ROW_HEIGHT, nextFreeSlot, resolve, type Placed } from '#lib/dashboard/layout.ts';
	import type { Chart, Layout } from '#lib/types/api.ts';

	let {
		charts,
		project,
		runs,
		colors,
		onlayout
	}: {
		charts: Chart[];
		project: string;
		runs: string[];
		colors: Map<string, string>;
		onlayout: (changes: Placed[]) => void;
	} = $props();

	const PADDING = 24;
	const GAP_X = 32;
	const GAP_Y = 40;
	const POINTS_PER_COL = 30;

	interface Drag {
		id: string;
		mode: 'move' | 'resize';
		x0: number;
		y0: number;
		start: Layout;
		dx: number;
		dy: number;
	}

	let viewport = $state(Infinity);
	let gridWidth = $state(0);
	let drag = $state<Drag | null>(null);
	let preview = $state<Placed[] | null>(null);

	const narrow = $derived(viewport < 760);
	const committed = $derived(charts.map((c) => ({ id: c.id, layout: c.config.layout })));
	const layouts = $derived(new Map((preview ?? committed).map((p) => [p.id, p.layout])));
	const addSlot = $derived(nextFreeSlot([...layouts.values()], 2, 1));
	const ordered = $derived(
		[...charts].sort(
			(a, b) => a.config.layout.y - b.config.layout.y || a.config.layout.x - b.config.layout.x
		)
	);
	const colWidth = $derived((gridWidth - 2 * PADDING - GAP_X * (COLS - 1)) / COLS);

	const area = (l: Layout) => `${l.y + 1} / ${l.x + 1} / span ${l.h} / span ${l.w}`;
	const pixels = (l: Layout) => ({
		width: l.w * colWidth + (l.w - 1) * GAP_X,
		height: l.h * ROW_HEIGHT + (l.h - 1) * GAP_Y
	});

	function start(event: PointerEvent, chart: Chart, mode: Drag['mode']) {
		if (narrow || event.button !== 0) return;
		if (mode === 'move' && !(event.target as Element).closest('[data-drag-handle]')) return;
		event.preventDefault();
		event.stopPropagation();
		drag = {
			id: chart.id,
			mode,
			x0: event.clientX,
			y0: event.clientY,
			start: chart.config.layout,
			dx: 0,
			dy: 0
		};
		preview = committed;
	}

	function move(event: PointerEvent) {
		if (!drag) return;
		drag.dx = event.clientX - drag.x0;
		drag.dy = event.clientY - drag.y0;
		const cols = Math.round(drag.dx / (colWidth + GAP_X));
		const rows = Math.round(drag.dy / (ROW_HEIGHT + GAP_Y));
		const { start } = drag;
		const target =
			drag.mode === 'move'
				? { ...start, x: start.x + cols, y: start.y + rows }
				: { ...start, w: start.w + cols, h: start.h + rows };
		preview = resolve(committed, drag.id, target);
	}

	function end() {
		if (!drag || !preview) return;
		const before = new Map(committed.map((p) => [p.id, p.layout]));
		const changes = preview.filter(({ id, layout }) => {
			const old = before.get(id);
			return !old || (['x', 'y', 'w', 'h'] as const).some((k) => old[k] !== layout[k]);
		});
		drag = null;
		preview = null;
		if (changes.length) onlayout(changes);
	}
</script>

<svelte:window
	bind:innerWidth={viewport}
	onpointermove={move}
	onpointerup={end}
	onpointercancel={end}
/>

{#if narrow}
	<main class="flex flex-col gap-10 p-6">
		{#each ordered as chart (chart.id)}
			<div style:height="{chart.config.layout.h * ROW_HEIGHT}px">
				<MetricChart
					config={chart.config}
					{project}
					{runs}
					{colors}
					points={chart.config.layout.w * POINTS_PER_COL}
				/>
			</div>
		{/each}
		<div style:height="{ROW_HEIGHT}px"><AddChartCard /></div>
	</main>
{:else}
	<main
		bind:clientWidth={gridWidth}
		class={['grid', drag && 'select-none']}
		style:grid-template-columns="repeat({COLS}, minmax(0, 1fr))"
		style:grid-auto-rows="{ROW_HEIGHT}px"
		style:gap="{GAP_Y}px {GAP_X}px"
		style:padding="{PADDING}px"
	>
		{#if drag}
			{@const target = layouts.get(drag.id)}
			{#if target}
				<div class="rounded bg-surface" style:grid-area={area(target)}></div>
			{/if}
		{/if}

		{#each charts as chart (chart.id)}
			{@const dragging = drag?.id === chart.id}
			{@const layout = dragging ? drag!.start : (layouts.get(chart.id) ?? chart.config.layout)}
			{@const size = dragging && drag!.mode === 'resize' ? pixels(layout) : null}
			<div
				role="group"
				class={['cell group relative min-w-0 hover:z-10', dragging && 'z-30']}
				style:grid-area={area(layout)}
				style:transform={dragging && drag!.mode === 'move'
					? `translate(${drag!.dx}px, ${drag!.dy}px)`
					: null}
				style:width={size ? `${Math.max(colWidth, size.width + drag!.dx)}px` : null}
				style:height={size ? `${Math.max(ROW_HEIGHT, size.height + drag!.dy)}px` : null}
				onpointerdown={(e) => start(e, chart, 'move')}
			>
				<MetricChart
					config={chart.config}
					{project}
					{runs}
					{colors}
					points={chart.config.layout.w * POINTS_PER_COL}
				/>
				<button
					type="button"
					aria-label="Resize {chart.config.title}"
					class="absolute -right-2 -bottom-2 flex size-4 cursor-nwse-resize touch-none items-center justify-center border-0 bg-transparent p-0 text-crosshair opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
					onpointerdown={(e) => start(e, chart, 'resize')}
				>
					<svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
						<path d="M9 1 L1 9 M9 5 L5 9" fill="none" stroke="currentColor" stroke-width="1.3" />
					</svg>
				</button>
			</div>
		{/each}

		<div style:grid-area={area(addSlot)}><AddChartCard /></div>
	</main>
{/if}

<style>
	.cell :global([data-drag-handle]) {
		cursor: grab;
		touch-action: none;
	}

	.cell:active :global([data-drag-handle]) {
		cursor: grabbing;
	}
</style>
