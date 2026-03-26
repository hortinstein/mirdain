import { useEffect, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, Trash2, GitBranch, Loader2 } from 'lucide-react'

import { FlowEditor } from './components/FlowEditor'
import { AiNodeModal } from './components/AiNodeModal'
import { api } from './api/client'
import { useStore } from './store'
import type { Pipeline, WsEvent } from './types'

export default function App() {
  const queryClient = useQueryClient()
  const { data: pipelines = [], isLoading } = useQuery({
    queryKey: ['pipelines'],
    queryFn: api.listPipelines,
  })

  const setPipelines = useStore((s) => s.setPipelines)
  const activePipelineId = useStore((s) => s.activePipelineId)
  const setActivePipelineId = useStore((s) => s.setActivePipelineId)
  const handleWsEvent = useStore((s) => s.handleWsEvent)
  const aiModalOpen = useStore((s) => s.aiModalOpen)
  const setAiModalOpen = useStore((s) => s.setAiModalOpen)

  const [creating, setCreating] = useState(false)
  const [newName, setNewName] = useState('')

  // Sync store
  useEffect(() => {
    setPipelines(pipelines)
  }, [pipelines, setPipelines])

  // WebSocket
  useEffect(() => {
    const proto = location.protocol === 'https:' ? 'wss' : 'ws'
    const ws = new WebSocket(`${proto}://${location.host}/ws`)
    ws.onmessage = (msg) => {
      try {
        const event: WsEvent = JSON.parse(msg.data)
        handleWsEvent(event)
        if (event.type === 'pipeline_status') {
          queryClient.invalidateQueries({ queryKey: ['pipelines'] })
        }
        if (event.type === 'node_types_updated') {
          queryClient.invalidateQueries({ queryKey: ['node-types'] })
        }
      } catch {}
    }
    ws.onerror = (e) => console.warn('WS error', e)
    return () => ws.close()
  }, [handleWsEvent, queryClient])

  const storePipelines = useStore((s) => s.pipelines)
  const activePipeline = storePipelines.find((p) => p.id === activePipelineId)

  const createMutation = useMutation({
    mutationFn: (name: string) => api.createPipeline({ name, nodes: [], edges: [] }),
    onSuccess: (p) => {
      queryClient.invalidateQueries({ queryKey: ['pipelines'] })
      setActivePipelineId(p.id)
      setCreating(false)
      setNewName('')
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deletePipeline(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: ['pipelines'] })
      if (activePipelineId === id) setActivePipelineId(null)
    },
  })

  const handleUpdate = (updated: Pipeline) => {
    useStore.getState().upsertPipeline(updated)
    queryClient.invalidateQueries({ queryKey: ['pipelines'] })
  }

  return (
    <div className="flex h-screen bg-canvas text-white overflow-hidden">
      {aiModalOpen && <AiNodeModal onClose={() => setAiModalOpen(false)} />}
      {/* Sidebar */}
      <div className="w-56 flex-shrink-0 border-r border-border bg-panel flex flex-col">
        {/* Logo */}
        <div className="p-4 border-b border-border">
          <div className="flex items-center gap-2">
            <GitBranch className="w-5 h-5 text-accent" />
            <span className="font-bold text-white tracking-tight">Mirdain</span>
          </div>
          <p className="text-[10px] text-gray-500 mt-0.5">Docker Flow Processor</p>
        </div>

        {/* Pipeline list */}
        <div className="flex-1 overflow-y-auto p-2">
          <div className="flex items-center justify-between mb-2 px-1">
            <span className="text-[10px] font-semibold text-gray-500 uppercase tracking-wider">
              Pipelines
            </span>
            <button
              onClick={() => setCreating(true)}
              className="text-gray-400 hover:text-accent transition-colors"
              title="New pipeline"
            >
              <Plus className="w-4 h-4" />
            </button>
          </div>

          {isLoading && (
            <div className="flex justify-center py-4">
              <Loader2 className="w-4 h-4 animate-spin text-gray-500" />
            </div>
          )}

          {storePipelines.map((p) => (
            <div
              key={p.id}
              onClick={() => setActivePipelineId(p.id)}
              className={`
                flex items-center justify-between px-2 py-1.5 rounded-lg cursor-pointer group mb-0.5
                ${activePipelineId === p.id
                  ? 'bg-accent/20 text-white'
                  : 'text-gray-400 hover:bg-white/5 hover:text-white'}
              `}
            >
              <div className="flex items-center gap-1.5 min-w-0">
                <span
                  className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${
                    p.status === 'running' ? 'bg-green-400' : 'bg-gray-600'
                  }`}
                />
                <span className="text-xs truncate">{p.name}</span>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation()
                  if (confirm(`Delete "${p.name}"?`)) deleteMutation.mutate(p.id)
                }}
                className="opacity-0 group-hover:opacity-100 text-gray-500 hover:text-red-400 transition-all"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          ))}

          {storePipelines.length === 0 && !isLoading && (
            <p className="text-xs text-gray-600 px-2 py-2">No pipelines yet.</p>
          )}
        </div>

        {/* Create form */}
        {creating && (
          <div className="p-3 border-t border-border space-y-2">
            <input
              autoFocus
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && newName.trim())
                  createMutation.mutate(newName.trim())
                if (e.key === 'Escape') setCreating(false)
              }}
              placeholder="Pipeline name…"
              className="w-full bg-canvas border border-border rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-accent"
            />
            <div className="flex gap-1">
              <button
                onClick={() => newName.trim() && createMutation.mutate(newName.trim())}
                className="flex-1 bg-accent hover:bg-accent-hover text-white text-xs py-1 rounded"
              >
                Create
              </button>
              <button
                onClick={() => setCreating(false)}
                className="text-gray-400 hover:text-white text-xs px-2 py-1 rounded border border-border"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Main area */}
      <div className="flex-1 overflow-hidden">
        {activePipeline ? (
          <FlowEditor
            key={activePipeline.id}
            pipeline={activePipeline}
            onUpdate={handleUpdate}
          />
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-center">
            <GitBranch className="w-16 h-16 text-gray-700 mb-4" />
            <h2 className="text-xl font-semibold text-gray-400 mb-2">
              No pipeline selected
            </h2>
            <p className="text-sm text-gray-600 max-w-xs">
              Create a new pipeline from the sidebar, then drag nodes from the
              library onto the canvas and connect them.
            </p>
            <button
              onClick={() => setCreating(true)}
              className="mt-4 flex items-center gap-2 bg-accent hover:bg-accent-hover text-white text-sm px-4 py-2 rounded-lg"
            >
              <Plus className="w-4 h-4" />
              New Pipeline
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
