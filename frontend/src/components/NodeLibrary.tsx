import { useQuery } from '@tanstack/react-query'
import { Package, ChevronRight } from 'lucide-react'
import { api } from '../api/client'
import type { NodeType } from '../types'

interface NodeLibraryProps {
  onDragStart: (event: React.DragEvent, nodeType: NodeType) => void
}

export function NodeLibrary({ onDragStart }: NodeLibraryProps) {
  const { data: nodeTypes = [], isLoading } = useQuery({
    queryKey: ['node-types'],
    queryFn: api.listNodeTypes,
  })

  return (
    <div className="h-full flex flex-col">
      <div className="p-3 border-b border-border">
        <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Node Library
        </h2>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {isLoading && (
          <p className="text-xs text-gray-500 p-2">Loading...</p>
        )}
        {nodeTypes.map((nt) => (
          <div
            key={nt.id}
            draggable
            onDragStart={(e) => onDragStart(e, nt)}
            className="
              flex items-start gap-2 p-2 rounded-lg border border-border cursor-grab
              bg-panel hover:border-accent hover:bg-accent/5 transition-all
              active:cursor-grabbing group
            "
          >
            <Package className="w-4 h-4 text-accent mt-0.5 flex-shrink-0" />
            <div className="min-w-0">
              <div className="text-sm font-medium text-white">{nt.name}</div>
              <div className="text-xs text-gray-500 truncate">{nt.description}</div>
              <div className="flex gap-2 mt-1">
                {nt.input_ports.length > 0 && (
                  <span className="text-[10px] text-blue-400">
                    {nt.input_ports.length} in
                  </span>
                )}
                {nt.output_ports.length > 0 && (
                  <span className="text-[10px] text-emerald-400">
                    {nt.output_ports.length} out
                  </span>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="p-3 border-t border-border">
        <p className="text-[10px] text-gray-600 text-center">
          Drag nodes onto the canvas
        </p>
      </div>
    </div>
  )
}
