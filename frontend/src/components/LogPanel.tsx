import { useStore } from '../store'

interface LogPanelProps {
  pipelineId: string
}

const kindColor: Record<string, string> = {
  exec: 'text-gray-400',
  build: 'text-purple-300',
}

const kindBadge: Record<string, string> = {
  exec: 'text-gray-600',
  build: 'bg-purple-900/40 text-purple-400 px-1 rounded',
}

export function LogPanel({ pipelineId }: LogPanelProps) {
  const logs = useStore((s) =>
    s.logs.filter(
      (l) =>
        l.kind === 'build' ||
        l.pipelineId === pipelineId ||
        l.pipelineId === ''
    )
  )

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-y-auto p-2 space-y-0.5 font-mono text-xs">
        {logs.length === 0 && (
          <span className="text-gray-600 p-1 block">No log entries yet.</span>
        )}
        {logs.map((l) => (
          <div key={l.id} className={`flex gap-2 ${kindColor[l.kind]}`}>
            <span className="text-gray-600 flex-shrink-0 tabular-nums">
              {new Date(l.ts).toLocaleTimeString()}
            </span>
            <span className={`flex-shrink-0 ${kindBadge[l.kind]}`}>
              {l.kind === 'build' ? 'build' : 'exec'}
            </span>
            <span className="break-all">{l.message}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
