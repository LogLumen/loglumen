<script lang="ts">
	import { onMount } from 'svelte';
	import AppHeader from '#lib/components/AppHeader.svelte';
	import DashboardGrid from '#lib/components/DashboardGrid.svelte';
	import RunsPanel from '#lib/components/RunsPanel.svelte';
	import { listProjects, listRuns, loadDashboard, saveChart } from '#lib/api/dashboards.ts';
	import { stream } from '#lib/api/stream.svelte.ts';
	import type { Placed } from '#lib/dashboard/layout.ts';
	import { runColor } from '#lib/dashboard/series.ts';
	import type { Chart } from '#lib/types/api.ts';

	let projects = $state<string[]>([]);
	let project = $state<string>();
	let runs = $state<string[]>([]);
	let selected = $state<string[]>([]);
	let charts = $state<Chart[]>([]);
	let panelOpen = $state(true);
	let loaded = $state(false);
	let ready = $state(false);
	let error = $state<string | null>(null);

	const colors = $derived(new Map(runs.map((run, i) => [run, runColor(i)])));

	onMount(() => {
		stream.connect();
		listProjects()
			.then((list) => {
				projects = list.map((p) => p.name);
				project = projects[0];
			})
			.catch((e: Error) => (error = e.message))
			.finally(() => (loaded = true));
	});

	$effect(() => {
		const name = project;
		if (!name) return;
		ready = false;
		let stale = false;
		Promise.all([listRuns(name), loadDashboard(name)])
			.then(([runList, dashboard]) => {
				if (stale) return;
				runs = runList.map((r) => r.name);
				selected = [...runs];
				charts = dashboard.charts;
				ready = true;
				error = null;
			})
			.catch((e: Error) => {
				if (!stale) error = e.message;
			});
		return () => {
			stale = true;
		};
	});

	function onlayout(changes: Placed[]) {
		const layouts = new Map(changes.map((c) => [c.id, c.layout]));
		charts = charts.map((c) => {
			const layout = layouts.get(c.id);
			return layout ? { ...c, config: { ...c.config, layout } } : c;
		});
		for (const chart of charts) {
			if (layouts.has(chart.id)) {
				saveChart(chart.id, $state.snapshot(chart.config)).catch((e: Error) => (error = e.message));
			}
		}
	}
</script>

<svelte:head>
	<title>LogLumen — {project ?? 'Run dashboard'}</title>
</svelte:head>

<div class="flex min-h-screen flex-col bg-bg text-fg">
	<AppHeader
		{projects}
		bind:project
		status={stream.status}
		runCount={selected.length}
		{panelOpen}
		ontogglepanel={() => (panelOpen = !panelOpen)}
	/>

	<div class="flex flex-1 flex-wrap items-stretch">
		<div class="flex min-w-0 flex-[999_1_560px] flex-col">
			{#if error}
				<p role="alert" class="mx-6 mt-4 mb-0 font-mono text-xs text-err">{error}</p>
			{/if}

			{#if loaded && !projects.length}
				{@render empty('No projects yet', 'Log a run with the loglumen client to see it here.')}
			{:else if ready && !selected.length}
				{@render empty(
					'No runs selected',
					'Tick one or more runs in the Runs panel to plot their metrics.'
				)}
			{:else if ready && project}
				<DashboardGrid {charts} {project} runs={selected} {colors} {onlayout} />
			{/if}
		</div>

		{#if panelOpen && ready}
			<RunsPanel {runs} bind:selected {colors} onhide={() => (panelOpen = false)} />
		{/if}
	</div>
</div>

{#snippet empty(title: string, hint: string)}
	<div class="m-6 flex flex-col items-center gap-1.5 rounded bg-surface px-6 py-16 text-center">
		<span class="font-mono text-[13px] text-fg">{title}</span>
		<span class="text-[13px] text-muted">{hint}</span>
	</div>
{/snippet}
