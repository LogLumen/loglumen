import { api } from './client.ts';
import { SEED_CHARTS } from '#lib/dashboard/layout.ts';
import type { Chart, ChartConfig, Dashboard, Project, Run } from '#lib/types/api.ts';

const path = (name: string) => encodeURIComponent(name);

export const listProjects = () => api<Project[]>('/projects');

export const listRuns = (project: string) => api<Run[]>(`/projects/${path(project)}/runs`);

/** The project's oldest dashboard, created and seeded with the default charts if there is none. */
export async function loadDashboard(
	project: string
): Promise<{ dashboard: Dashboard; charts: Chart[] }> {
	const [existing] = await api<Dashboard[]>(`/projects/${path(project)}/dashboards`);
	if (existing) {
		return { dashboard: existing, charts: await api<Chart[]>(`/dashboards/${existing.id}/charts`) };
	}

	const dashboard = await api<Dashboard>(`/projects/${path(project)}/dashboards`, {
		method: 'POST',
		body: JSON.stringify({ config: {} })
	});
	const charts: Chart[] = [];
	for (const config of SEED_CHARTS) {
		charts.push(
			await api<Chart>(`/dashboards/${dashboard.id}/charts`, {
				method: 'POST',
				body: JSON.stringify({ config })
			})
		);
	}
	return { dashboard, charts };
}

export const saveChart = (id: string, config: ChartConfig) =>
	api<Chart>(`/charts/${id}`, { method: 'PUT', body: JSON.stringify({ config }) });
