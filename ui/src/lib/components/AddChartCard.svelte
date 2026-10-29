<div
	aria-label="Add chart"
	aria-disabled="true"
	class="add flex h-full flex-col items-center justify-center gap-3.5 rounded border border-dashed border-border bg-transparent p-6 text-subtle"
>
	<svg width="56" height="56" viewBox="0 0 56 56" aria-hidden="true">
		<path
			d="M28 8 V48 M8 28 H48"
			fill="none"
			stroke="currentColor"
			stroke-width="2.5"
			stroke-linecap="square"
		/>
	</svg>
	<span class="flex flex-col items-center gap-1">
		<span class="font-mono text-[13px]">Add chart</span>
		<span class="text-xs text-muted">Pick a metric_key, x-axis and range</span>
	</span>
</div>

<style>
	.add:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}
</style>
