import { create } from 'zustand'
import type { NodeMetrics, Pipeline, WsEvent } from '../types'

export interface LogEntry {
  id: number
  pipelineId: string
  nodeTypeId?: string
  message: string
  kind: 'exec' | 'build'
  ts: number
}

interface NodeStatusMap {
  [nodeId: string]: string
}

interface MirdainStore {
  // Pipeline list
  pipelines: Pipeline[]
  setPipelines: (p: Pipeline[]) => void
  upsertPipeline: (p: Pipeline) => void
  removePipeline: (id: string) => void

  // Currently open pipeline ID
  activePipelineId: string | null
  setActivePipelineId: (id: string | null) => void

  // Selected node in the canvas
  selectedNodeId: string | null
  setSelectedNodeId: (id: string | null) => void

  // Real-time node statuses from WS
  nodeStatuses: NodeStatusMap
  // Per-node metrics from WS
  nodeMetrics: Record<string, NodeMetrics>
  // Logs (both exec and build)
  logs: LogEntry[]
  logCounter: number

  // Build status per node type id
  buildStatuses: Record<string, 'building' | 'ready' | 'error'>

  handleWsEvent: (event: WsEvent) => void

  // Trigger input JSON string (for the trigger modal)
  triggerInput: string
  setTriggerInput: (v: string) => void

  // AI generator modal open state
  aiModalOpen: boolean
  setAiModalOpen: (open: boolean) => void
}

export const useStore = create<MirdainStore>((set) => ({
  pipelines: [],
  setPipelines: (pipelines) => set({ pipelines }),
  upsertPipeline: (p) =>
    set((s) => ({
      pipelines: s.pipelines.some((x) => x.id === p.id)
        ? s.pipelines.map((x) => (x.id === p.id ? p : x))
        : [...s.pipelines, p],
    })),
  removePipeline: (id) =>
    set((s) => ({ pipelines: s.pipelines.filter((p) => p.id !== id) })),

  activePipelineId: null,
  setActivePipelineId: (id) => set({ activePipelineId: id, selectedNodeId: null }),

  selectedNodeId: null,
  setSelectedNodeId: (id) => set({ selectedNodeId: id }),

  nodeStatuses: {},
  nodeMetrics: {},
  logs: [],
  logCounter: 0,
  buildStatuses: {},

  handleWsEvent: (event) => {
    set((s) => {
      const counter = s.logCounter + 1

      if (event.type === 'node_status' && event.nodeId && event.status) {
        return { nodeStatuses: { ...s.nodeStatuses, [event.nodeId]: event.status } }
      }

      if (event.type === 'pipeline_status' && event.pipelineId && event.status) {
        return {
          pipelines: s.pipelines.map((p) =>
            p.id === event.pipelineId
              ? { ...p, status: event.status as Pipeline['status'] }
              : p
          ),
        }
      }

      if (event.type === 'execution_log' && event.message) {
        const entry: LogEntry = {
          id: counter,
          pipelineId: event.pipelineId ?? '',
          message: event.message,
          kind: 'exec',
          ts: Date.now(),
        }
        return { logCounter: counter, logs: [...s.logs.slice(-499), entry] }
      }

      if (event.type === 'metrics_update' && event.nodeId && event.metrics) {
        return {
          nodeMetrics: { ...s.nodeMetrics, [event.nodeId]: event.metrics },
        }
      }

      if (event.type === 'build_log' && event.message) {
        const entry: LogEntry = {
          id: counter,
          pipelineId: '',
          nodeTypeId: event.nodeTypeId,
          message: event.message,
          kind: 'build',
          ts: Date.now(),
        }
        return {
          logCounter: counter,
          logs: [...s.logs.slice(-499), entry],
        }
      }

      if (event.type === 'build_complete' && event.nodeTypeId) {
        return {
          buildStatuses: { ...s.buildStatuses, [event.nodeTypeId]: 'ready' },
        }
      }

      if (event.type === 'build_error' && event.nodeTypeId) {
        return {
          buildStatuses: { ...s.buildStatuses, [event.nodeTypeId]: 'error' },
        }
      }

      return {}
    })
  },

  triggerInput: '{"value": "hello world"}',
  setTriggerInput: (v) => set({ triggerInput: v }),

  aiModalOpen: false,
  setAiModalOpen: (open) => set({ aiModalOpen: open }),
}))
