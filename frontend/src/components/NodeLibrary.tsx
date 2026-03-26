import { useQuery, useQueryClient } from '@tanstack/react-query'
import { Package, Sparkles, Loader2, CheckCircle2, AlertCircle } from 'lucide-react'
import { api } from '../api/client'
import { useStore } from '../store'
import type { NodeType } from '../types'

interface NodeLibraryProps {
  onDragStart: (event: React.DragEvent, nodeType: NodeType) => void
}

const BUILD_ICON = {
  ready: null,
  building: <Loader2 className="w-3 h-3 text-yellow-400 animate-spin flex-shrink-0" />,
  error: <AlertCircle className="w-3 h-3 text-red-400 flex-shrink-0" />,
}

export function NodeLibrary({ onDragStart }: NodeLibraryProps) {
  const queryClient = useQueryClient()
  const setAiModalOpen = useStore((s) => s.setAiModalOpen)
  const buildStatuses = useStore((s) => s.buildStatuses)

  // Refresh node types when server notifies
  // (handled in App.tsx via node_types_updated WS event)

  const { data: nodeTypes = [], isLoading } = useQuery({
    queryKey: ['node-types'],
    queryFn: api.listNodeTypes,
  })

  const builtIns = nodeTypes.filter((n) => !n.ai_prompt)
  const aiNodes = nodeTypes.filter((n) => !!n.ai_prompt)

  return (
    <div className="h-full flex flex-col">
      <div className="p-3 border-b border-border flex items-center justify-between">
        <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Node Library
        </h2>
        <button
          onClick={() => setAiModalOpen(true)}
          className="flex items-center gap-1 text-[10px] text-accent hover:text-accent-hover bg-accent/10 hover:bg-accent/20 px-1.5 py-0.5 rounded transition-colors"
          title="Generate a new node with AI"
        >
          <Sparkles className="w-3 h-3" />
          AI
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-3">
        {isLoading && (
          <div className="flex justify-center py-4">
            <Loader2 className="w-4 h-4 animate-spin text-gray-500" />
          </div>
        )}

        {/* Built-in nodes */}
        <NodeGroup title="Built-in" nodes={builtIns} onDragStart={onDragStart} buildStatuses={buildStatuses} />

        {/* AI-generated nodes */}
        {aiNodes.length > 0 && (
          <NodeGroup title="AI Generated" nodes={aiNodes} onDragStart={onDragStart} buildStatuses={buildStatuses} ai />
        )}
      </div>

      <div className="p-3 border-t border-border">
        <p className="text-[10px] text-gray-600 text-center">
          Drag onto canvas · click AI to generate
        </p>
      </div>
    </div>
  )
}

function NodeGroup({
  title,
  nodes,
  onDragStart,
  buildStatuses,
  ai = false,
}: {
  title: string
  nodes: NodeType[]
  onDragStart: (e: React.DragEvent, nt: NodeType) => void
  buildStatuses: Record<string, 'building' | 'ready' | 'error'>
  ai?: boolean
}) {
  if (nodes.length === 0) return null

  return (
    <div>
      <p className="text-[10px] text-gray-600 uppercase tracking-wider px-1 mb-1">{title}</p>
      <div className="space-y-1">
        {nodes.map((nt) => {
          const status = buildStatuses[nt.id] ?? nt.build_status
          const isBuilding = status === 'building'
          const isError = status === 'error'
          return (
            <div
              key={nt.id}
              draggable={!isBuilding}
              onDragStart={(e) => !isBuilding && onDragStart(e, nt)}
              className={`
                flex items-start gap-2 p-2 rounded-lg border border-border
                bg-panel transition-all
                ${isBuilding
                  ? 'opacity-60 cursor-not-allowed'
                  : 'cursor-grab hover:border-accent hover:bg-accent/5 active:cursor-grabbing'}
                ${isError ? 'border-red-800' : ''}
              `}
            >
              {ai ? (
                <Sparkles className="w-4 h-4 text-purple-400 mt-0.5 flex-shrink-0" />
              ) : (
                <Package className="w-4 h-4 text-accent mt-0.5 flex-shrink-0" />
              )}
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1">
                  <span className="text-xs font-medium text-white truncate">{nt.name}</span>
                  {BUILD_ICON[status as keyof typeof BUILD_ICON]}
                </div>
                <div className="text-[10px] text-gray-500 truncate">{nt.description}</div>
                <div className="flex gap-2 mt-0.5">
                  {nt.input_ports.length > 0 && (
                    <span className="text-[9px] text-blue-400">{nt.input_ports.length} in</span>
                  )}
                  {nt.output_ports.length > 0 && (
                    <span className="text-[9px] text-emerald-400">{nt.output_ports.length} out</span>
                  )}
                  {isBuilding && <span className="text-[9px] text-yellow-400">building…</span>}
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
