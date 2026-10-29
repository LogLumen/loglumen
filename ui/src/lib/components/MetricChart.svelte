<script lang="ts">
	import { Area, Chart, Spline, Svg } from 'layerchart';
	import { stream } from '#lib/api/stream.svelte.ts';
	import { bucketAt, formatValue, formatX, type RunSeries } from '#lib/dashboard/series.ts';
	import type { ChartConfig, Query } from '#lib/types/api.ts';

	let {
		config,
		project,
		runs,
		colors,
		points
	}: {
		config: ChartConfig;
		project: string;
		runs: string[];
		colors: Map<string, string>;
		points: number;
	} = $props();

	const PAD = 8;

	let data = $state<RunSeries[]>([]);
	let error = $state<string | null>(null);
	let width = $state(0);
	let height = $state(0);
	let hoverX = $state<number | null>(null);

	$effect(() => {
		const names = $state.snapshot(runs);
		if (!names.length) {
			data = [];
			return;
		}
		const query = (run: string): Query => ({
			project_name: project,
			key: config.key,
			x: config.x === 'timestamp' ? 'timestamp' : { metric: config.x },
			select: config.select,
			filter: { type: 'run_in', runs: [run] },
			max_points: points
		});
		let stale = false;
		Promise.all(names.map((run) => stream.query(query(run))))
			.then((results) => {
				if (stale) return;
				data = names
					.map((run, i) => ({ run, buckets: results[i] }))
					.filter((s) => s.buckets.length);
				error = null;
			})
			.catch((e: Error) => {
				if (!stale) error = e.message;
			});
		return () => {
			stale = true;
		};
	});

	const xDomain = $derived.by((): [number, number] => {
		if (!data.length) return [0, 1];
		const lo = Math.min(...data.map((s) => s.buckets[0].x));
		const hi = Math.max(...data.map((s) => s.buckets[s.buckets.length - 1].x));
		return [lo, hi > lo ? hi : lo + 1];
	});

	const yDomain = $derived.by((): [number, number] => {
		let lo = Infinity;
		let hi = -Infinity;
		for (const { buckets } of data) {
			for (const b of buckets) {
				lo = Math.min(lo, b.min);
				hi = Math.max(hi, b.max);
			}
		}
		if (lo === Infinity) return [0, 1];
		let span = hi - lo;
		if (span < 1e-12) span = Math.abs(hi) * 0.02 + 1e-9;
		return [lo - span * 0.08, hi + span * 0.08];
	});

	const xPos = (v: number) => ((v - xDomain[0]) / (xDomain[1] - xDomain[0])) * 100;
	const yPos = (v: number) =>
		PAD + (1 - (v - yDomain[0]) / (yDomain[1] - yDomain[0])) * (height - 2 * PAD);

	const single = $derived(runs.length === 1);
	const last = $derived(data[0]?.buckets.at(-1));

	const hover = $derived.by(() => {
		if (hoverX === null || !data.length) return null;
		const at = hoverX;
		const rows = data.flatMap(({ run, buckets }) => {
			const bucket = bucketAt(buckets, at);
			return bucket ? [{ run, color: colors.get(run) ?? 'transparent', bucket }] : [];
		});
		const x = rows.length
			? rows.reduce(
					(best, r) => (Math.abs(r.bucket.x - at) < Math.abs(best - at) ? r.bucket.x : best),
					rows[0].bucket.x
				)
			: at;
		return { x, left: xPos(x), rows };
	});

	function onpointermove(event: PointerEvent) {
		if (!width) return;
		hoverX = xDomain[0] + (event.offsetX / width) * (xDomain[1] - xDomain[0]);
	}
</script>

<section aria-label={config.title} class="flex h-full min-w-0 flex-col gap-2.5 bg-bg">
	<div data-drag-handle class="flex items-start justify-between gap-3">
		<h2 class="m-0 min-w-0 flex-1 truncate font-mono text-[13px] font-medium">{config.title}</h2>
		{#if single}
			<div class="flex min-w-24 flex-col items-end gap-[3px]">
				<span class="font-mono text-xl text-fg tabular-nums"
					>{last ? formatValue(last.avg) : '–'}</span
				>
				<span class="text-xs text-muted">last bucket avg</span>
			</div>
		{/if}
	</div>

	<div class="flex min-h-0 flex-1 gap-2">
		<div class="relative min-w-0 flex-1" bind:clientWidth={width} bind:clientHeight={height}>
			<div aria-hidden="true" class="pointer-events-none absolute inset-0">
				<div class="absolute inset-x-0 h-px bg-grid" style:top="{PAD}px"></div>
				<div class="absolute inset-x-0 top-1/2 h-px bg-grid"></div>
				<div class="absolute inset-x-0 h-px bg-grid" style:bottom="{PAD}px"></div>
			</div>

			{#if data.length}
				<div aria-hidden="true" class="absolute inset-0">
					<Chart
						x="x"
						y="avg"
						{xDomain}
						{yDomain}
						padding={{ top: PAD, bottom: PAD, left: 0, right: 0 }}
						tooltipContext={false}
					>
						<Svg>
							{#each data as { run, buckets } (run)}
								<Area
									data={buckets}
									x="x"
									y0="min"
									y1="max"
									fill={colors.get(run)}
									opacity={single ? 0.12 : 0.06}
								/>
								<Spline
									data={buckets}
									x="x"
									y="avg"
									stroke={colors.get(run)}
									stroke-width={1.5}
									stroke-linejoin="round"
								/>
							{/each}
						</Svg>
					</Chart>
				</div>
			{:else if runs.length}
				<div class="absolute inset-0 flex items-center justify-center font-mono text-xs text-muted">
					{error ?? 'No data'}
				</div>
			{/if}

			{#if hover}
				<div
					class="pointer-events-none absolute inset-y-0 w-px bg-crosshair"
					style:left="{hover.left}%"
				></div>
				{#each hover.rows as row (row.run)}
					<div
						class="pointer-events-none absolute -mt-[5px] -ml-1 size-[7px] rounded-full border-[1.5px] bg-bg"
						style:left="{xPos(row.bucket.x)}%"
						style:top="{yPos(row.bucket.avg)}px"
						style:border-color={row.color}
					></div>
				{/each}
			{/if}

			<div
				role="presentation"
				class="absolute inset-0"
				{onpointermove}
				onpointerleave={() => (hoverX = null)}
			></div>

			{#if hover}
				<div
					role="tooltip"
					class="pointer-events-none absolute top-1.5 z-20 flex flex-col gap-[5px] rounded border border-line bg-bg px-2.5 py-2 font-mono text-xs whitespace-nowrap tabular-nums shadow-[0_4px_12px_rgba(15,23,42,0.08)]"
					style:left={hover.left < 50 ? `calc(${hover.left}% + 12px)` : null}
					style:right={hover.left >= 50 ? `calc(${100 - hover.left}% + 12px)` : null}
				>
					<div class="pb-[5px] text-fg">{formatX(config.x, hover.x)}</div>
					<div class="tip-row text-[11px] text-muted">
						<span></span><span>run</span><span class="text-right">avg</span>
						<span class="text-right">min – max</span><span class="text-right">n</span>
					</div>
					{#each hover.rows as { run, color, bucket } (run)}
						<div class="tip-row items-center text-fg">
							<span aria-hidden="true" class="h-[3px] w-2.5" style:background={color}></span>
							<span class="truncate">{run}</span>
							<span class="text-right">{formatValue(bucket.avg)}</span>
							<span class="text-right text-subtle">
								{formatValue(bucket.min)} – {formatValue(bucket.max)}
							</span>
							<span class="text-right text-subtle">{bucket.count.toLocaleString('en-US')}</span>
						</div>
					{:else}
						<div class="tip-row text-fg">
							<span></span><span>no data</span><span class="text-right">–</span>
							<span class="text-right">–</span><span class="text-right">–</span>
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<div
			aria-hidden="true"
			class="relative w-[72px] flex-none font-mono text-[11px] text-muted tabular-nums"
		>
			<span class="absolute left-0 -translate-y-1/2" style:top="{PAD}px"
				>{formatValue(yDomain[1])}</span
			>
			<span class="absolute top-1/2 left-0 -translate-y-1/2">
				{formatValue((yDomain[0] + yDomain[1]) / 2)}
			</span>
			<span class="absolute left-0 translate-y-1/2" style:bottom="{PAD}px"
				>{formatValue(yDomain[0])}</span
			>
		</div>
	</div>

	<div
		class={['flex justify-between gap-2 font-mono text-[11px] text-muted', width >= 160 && 'mr-20']}
	>
		{#if data.length}
			<span>{formatX(config.x, xDomain[0])}</span>
			{#if width >= 240}
				<span>{formatX(config.x, (xDomain[0] + xDomain[1]) / 2)}</span>
			{/if}
			<span>{formatX(config.x, xDomain[1])}</span>
		{:else}
			<span>&nbsp;</span>
		{/if}
	</div>
</section>

<style>
	.tip-row {
		display: grid;
		grid-template-columns: 10px 128px 72px 136px 48px;
		gap: 10px;
	}
</style>
