<script lang="ts">
	import type { StreamStatus } from '#lib/api/stream.svelte.ts';

	let {
		projects,
		project = $bindable(),
		status,
		runCount,
		panelOpen,
		ontogglepanel
	}: {
		projects: string[];
		project: string | undefined;
		status: StreamStatus;
		runCount: number;
		panelOpen: boolean;
		ontogglepanel: () => void;
	} = $props();

	const STATUS = {
		connecting: { color: 'var(--color-warn)', label: 'Connecting…' },
		connected: { color: 'var(--color-ok)', label: 'Connected' },
		reconnecting: { color: 'var(--color-warn)', label: 'Reconnecting…' },
		disconnected: { color: 'var(--color-err)', label: 'Disconnected' }
	};
</script>

<header class="flex flex-wrap items-center gap-x-7 gap-y-3 px-6 pt-4 pb-2">
	<div class="flex items-center gap-2.5">
		<svg width="18" height="18" viewBox="0 0 18 18" aria-hidden="true">
			<path
				d="M1 13 L5 8 L9 11 L13 4 L17 7"
				fill="none"
				stroke="var(--color-accent)"
				stroke-width="2"
				stroke-linejoin="miter"
			/>
		</svg>
		<span class="text-[15px] font-semibold tracking-[0.01em]">LogLumen</span>
	</div>

	<select
		aria-label="Project"
		bind:value={project}
		class="h-10 min-w-[220px] rounded border border-border bg-bg px-2.5 font-mono text-[13px] text-fg"
	>
		{#each projects as name (name)}
			<option value={name}>{name}</option>
		{/each}
	</select>

	<div
		role="status"
		aria-live="polite"
		class="ml-auto flex items-center gap-2 font-mono text-xs text-subtle"
	>
		<span aria-hidden="true" class="size-2 rounded-full" style:background={STATUS[status].color}
		></span>
		<span>{STATUS[status].label}</span>
	</div>

	<button
		type="button"
		aria-expanded={panelOpen}
		aria-controls="runs-panel"
		onclick={ontogglepanel}
		class={[
			'flex h-9 cursor-pointer items-center gap-2 rounded border-0 px-3 font-mono text-xs text-accent-hover',
			panelOpen ? 'bg-active' : 'bg-transparent'
		]}
	>
		<svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
			<path
				d="M1.5 2.5 H14.5 V13.5 H1.5 Z M10 2.5 V13.5"
				fill="none"
				stroke="currentColor"
				stroke-width="1.3"
			/>
		</svg>
		<span>Runs · {runCount}</span>
	</button>
</header>
