import { memo } from 'react'
import { Handle, Position, NodeProps } from 'reactflow'
import { Container, AlertCircle, CheckCircle2, Loader2 } from 'lucide-react'
import { useStore } from '../store'
import type { FlowNode, Port } from '../types'

const statusColors: Record<string, string> = {
  processing: 'border-yellow-500 shadow-yellow-500/20',
  done: 'border-green-500 shadow-green-500/20',
  error: 'border-red-500 shadow-red-500/20',
  starting: 'border-blue-500 shadow-blue-500/20',
}

const StatusIcon = ({ status }: { status: string }) => {
  if (status === 'processing') return <Loader2 className="w-3 h-3 animate-spin text-yellow-400" />
  if (status === 'done') return <CheckCircle2 className="w-3 h-3 text-green-400" />
  if (status === 'error') return <AlertCircle className="w-3 h-3 text-red-400" />
  return null
}

export const ContainerNode = memo(({ id, data, selected }: NodeProps<FlowNode>) => {
  const nodeStatus = useStore((s) => s.nodeStatuses[id])
  const borderClass = nodeStatus ? statusColors[nodeStatus] ?? '' : ''

  return (
    <div
      className={`
        bg-panel border rounded-xl px-4 py-3 min-w-[180px] shadow-lg transition-all
        ${selected ? 'border-accent shadow-accent/20' : `border-border ${borderClass}`}
      `}
    >
      {/* Input handles */}
      {data.input_ports.map((port: Port, i: number) => (
        <Handle
          key={port.id}
          type="target"
          position={Position.Left}
          id={port.id}
          style={{ top: `${((i + 1) / (data.input_ports.length + 1)) * 100}%` }}
          className="!w-3 !h-3 !bg-blue-500 !border-2 !border-panel"
          title={port.label}
        />
      ))}

      {/* Node header */}
      <div className="flex items-center gap-2 mb-2">
        <Container className="w-4 h-4 text-accent flex-shrink-0" />
        <span className="text-sm font-semibold text-white truncate">{data.label}</span>
        {nodeStatus && <StatusIcon status={nodeStatus} />}
      </div>

      {/* Image tag */}
      <div className="text-xs text-gray-500 font-mono truncate max-w-[200px]">
        {data.image || <span className="text-red-400 italic">no image set</span>}
      </div>

      {/* Port labels */}
      {data.input_ports.length > 0 && (
        <div className="mt-2 space-y-1">
          {data.input_ports.map((p: Port) => (
            <div key={p.id} className="text-xs text-blue-400 text-left">
              ← {p.label}
            </div>
          ))}
        </div>
      )}
      {data.output_ports.length > 0 && (
        <div className="mt-1 space-y-1">
          {data.output_ports.map((p: Port) => (
            <div key={p.id} className="text-xs text-emerald-400 text-right">
              {p.label} →
            </div>
          ))}
        </div>
      )}

      {/* Output handles */}
      {data.output_ports.map((port: Port, i: number) => (
        <Handle
          key={port.id}
          type="source"
          position={Position.Right}
          id={port.id}
          style={{ top: `${((i + 1) / (data.output_ports.length + 1)) * 100}%` }}
          className="!w-3 !h-3 !bg-emerald-500 !border-2 !border-panel"
          title={port.label}
        />
      ))}
    </div>
  )
})

ContainerNode.displayName = 'ContainerNode'
