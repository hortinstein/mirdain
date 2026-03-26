import { useStore } from '../store'

interface LogPanelProps {
  pipelineId: string
}

export function LogPanel({ pipelineId }: LogPanelProps) {
  const logs = useStore((s) =>
    s.logs.filter((l) => l.pipelineId === pipelineId || l.pipelineId === '')
  )

  return (
    <div className="h-full flex flex-col">
      <div className="p-2 border-b border-border">
        <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Execution Log
        </span>
      </div>
      <div className="flex-1 overflow-y-auto p-2 space-y-0.5 font-mono text-xs">
        {logs.length === 0 && (
          <span className="text-gray-600">No log entries yet.</span>
        )}
        {logs.map((l) => (
          <div key={l.id} className="flex gap-2 text-gray-400">
            <span className="text-gray-600 flex-shrink-0">
              {new Date(l.ts).toLocaleTimeString()}
            </span>
            <span>{l.message}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
