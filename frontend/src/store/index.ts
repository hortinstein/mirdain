import { create } from 'zustand'
import type { Pipeline, WsEvent } from '../types'

interface LogEntry {
  id: number
  pipelineId: string
  message: string
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

  // Real-time statuses from WebSocket
  nodeStatuses: NodeStatusMap
  logs: LogEntry[]
  logCounter: number
  handleWsEvent: (event: WsEvent) => void

  // Trigger input JSON string (for the trigger modal)
  triggerInput: string
  setTriggerInput: (v: string) => void
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
  logs: [],
  logCounter: 0,

  handleWsEvent: (event) => {
    set((s) => {
      if (event.type === 'node_status' && event.nodeId && event.status) {
        return {
          nodeStatuses: { ...s.nodeStatuses, [event.nodeId]: event.status },
        }
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
          id: s.logCounter + 1,
          pipelineId: event.pipelineId ?? '',
          message: event.message,
          ts: Date.now(),
        }
        return {
          logCounter: s.logCounter + 1,
          logs: [...s.logs.slice(-199), entry],
        }
      }
      return {}
    })
  },

  triggerInput: '{"value": "hello world"}',
  setTriggerInput: (v) => set({ triggerInput: v }),
}))
