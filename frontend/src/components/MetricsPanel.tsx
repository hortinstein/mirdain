import { useStore } from '../store'
import type { FlowNode } from '../types'

function fmtBytes(b: number): string {
  if (b < 1024) return `${b}B`
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)}KB`
  return `${(b / 1024 / 1024).toFixed(2)}MB`
}

function fmtMs(ms: number): string {
  return ms < 1 ? '<1ms' : `${ms.toFixed(0)}ms`
}

interface MetricsPanelProps {
  nodes: FlowNode[]
}

export function MetricsPanel({ nodes }: MetricsPanelProps) {
  const nodeMetrics = useStore((s) => s.nodeMetrics)

  const rows = nodes.map((n) => ({
    node: n,
    m: nodeMetrics[n.id] ?? null,
  }))

  const hasAny = rows.some((r) => r.m !== null)

  return (
    <div className="h-full flex flex-col">
      <div className="p-2 border-b border-border flex items-center justify-between">
        <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Node Metrics
        </span>
        {hasAny && (
          <span className="text-[10px] text-gray-600">
            {rows.filter((r) => r.m).length} active
          </span>
        )}
      </div>

      {!hasAny ? (
        <div className="flex-1 flex items-center justify-center">
          <p className="text-xs text-gray-600">No data yet — trigger the pipeline to see metrics.</p>
        </div>
      ) : (
        <div className="flex-1 overflow-auto">
          <table className="w-full text-xs">
            <thead>
              <tr className="text-gray-500 border-b border-border">
                <th className="text-left px-3 py-1.5 font-medium">Node</th>
                <th className="text-right px-2 py-1.5 font-medium">Msgs</th>
                <th className="text-right px-2 py-1.5 font-medium">In</th>
                <th className="text-right px-2 py-1.5 font-medium">Out</th>
                <th className="text-right px-2 py-1.5 font-medium">Avg Lat</th>
                <th className="text-right px-3 py-1.5 font-medium">Errors</th>
              </tr>
            </thead>
            <tbody>
              {rows.map(({ node, m }) => (
                <tr
                  key={node.id}
                  className="border-b border-border/50 hover:bg-white/3 transition-colors"
                >
                  <td className="px-3 py-1.5 text-white truncate max-w-[120px]">{node.label}</td>
                  <td className="px-2 py-1.5 text-right text-gray-300">
                    {m ? m.messages_in.toLocaleString() : '–'}
                  </td>
                  <td className="px-2 py-1.5 text-right text-blue-400">
                    {m ? fmtBytes(m.bytes_in) : '–'}
                  </td>
                  <td className="px-2 py-1.5 text-right text-emerald-400">
                    {m ? fmtBytes(m.bytes_out) : '–'}
                  </td>
                  <td className="px-2 py-1.5 text-right text-yellow-400">
                    {m ? fmtMs(m.avg_latency_ms) : '–'}
                  </td>
                  <td className="px-3 py-1.5 text-right">
                    {m ? (
                      <span className={m.errors > 0 ? 'text-red-400' : 'text-gray-600'}>
                        {m.errors}
                      </span>
                    ) : (
                      '–'
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
