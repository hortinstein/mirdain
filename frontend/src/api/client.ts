import type { Pipeline, NodeType } from '../types'

const BASE = '/api'

async function req<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(`${res.status}: ${text}`)
  }
  if (res.status === 204) return undefined as T
  return res.json() as Promise<T>
}

export const api = {
  // Pipeline CRUD
  listPipelines: () => req<Pipeline[]>('/pipelines'),
  getPipeline: (id: string) => req<Pipeline>(`/pipelines/${id}`),
  createPipeline: (body: Partial<Pipeline>) =>
    req<Pipeline>('/pipelines', { method: 'POST', body: JSON.stringify(body) }),
  updatePipeline: (id: string, body: Partial<Pipeline>) =>
    req<Pipeline>(`/pipelines/${id}`, { method: 'PUT', body: JSON.stringify(body) }),
  deletePipeline: (id: string) => req<void>(`/pipelines/${id}`, { method: 'DELETE' }),

  // Execution
  deployPipeline: (id: string) =>
    req<{ status: string }>(`/pipelines/${id}/deploy`, { method: 'POST' }),
  stopPipeline: (id: string) =>
    req<{ status: string }>(`/pipelines/${id}/stop`, { method: 'POST' }),
  triggerPipeline: (id: string, data: unknown) =>
    req<{ outputs: unknown }>(`/pipelines/${id}/trigger`, {
      method: 'POST',
      body: JSON.stringify({ data }),
    }),

  // Node types
  listNodeTypes: () => req<NodeType[]>('/node-types'),
}
