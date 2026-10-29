<script lang="ts">
	let {
		runs,
		selected = $bindable(),
		colors,
		onhide
	}: {
		runs: string[];
		selected: string[];
		colors: Map<string, string>;
		onhide: () => void;
	} = $props();

	function toggle(run: string) {
		selected = selected.includes(run)
			? selected.filter((r) => r !== run)
			: runs.filter((r) => r === run || selected.includes(r));
	}
</script>

<aside
	id="runs-panel"
	aria-label="Runs"
	class="flex min-w-0 flex-[1_1_280px] flex-col border-l border-line bg-bg"
>
	<div class="flex items-center justify-between gap-3 px-4 pt-5 pb-3">
		<div class="flex flex-col gap-0.5">
			<h2 class="m-0 text-sm font-semibold">Runs</h2>
			<span class="text-xs text-muted">{selected.length} of {runs.length} selected</span>
		</div>
		<div class="flex items-center gap-1">
			<button type="button" class="text-btn" onclick={() => (selected = [...runs])}>All</button>
			<button type="button" class="text-btn" onclick={() => (selected = [])}>None</button>
			<button
				type="button"
				aria-label="Hide runs panel"
				onclick={onhide}
				class="flex size-8 cursor-pointer items-center justify-center rounded border-0 bg-transparent text-subtle"
			>
				<svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
					<path d="M6 3.5 L10.5 8 L6 12.5" fill="none" stroke="currentColor" stroke-width="1.5" />
				</svg>
			</button>
		</div>
	</div>

	<div class="flex flex-col gap-0.5 px-2">
		{#each runs as run (run)}
			{@const checked = selected.includes(run)}
			{@const color = colors.get(run)}
			<label
				class={[
					'flex min-h-[52px] cursor-pointer items-center gap-3 rounded px-2 py-1.5',
					checked ? 'bg-selected' : 'bg-transparent'
				]}
			>
				<input
					type="checkbox"
					{checked}
					onchange={() => toggle(run)}
					class="m-0 size-4 flex-none cursor-pointer"
					style:accent-color={color}
				/>
				<span
					aria-hidden="true"
					class="h-[3px] w-3.5 flex-none"
					style:background={color}
					style:opacity={checked ? 1 : 0.35}
				></span>
				<span
					class={['min-w-0 truncate font-mono text-[13px]', checked ? 'text-fg' : 'text-muted']}
				>
					{run}
				</span>
			</label>
		{/each}
	</div>

	<p class="m-0 p-4 text-xs leading-normal text-muted">
		Each chart uses its own x-axis: training step or timestamp.
	</p>
</aside>

<style>
	.text-btn {
		height: 32px;
		padding: 0 8px;
		background: transparent;
		color: var(--color-accent);
		border: 0;
		font-size: 12px;
		cursor: pointer;
	}

	.text-btn:hover {
		color: var(--color-accent-hover);
		text-decoration: underline;
	}
</style>
