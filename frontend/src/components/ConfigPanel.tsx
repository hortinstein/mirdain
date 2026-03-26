import { useState, useEffect } from 'react'
import { X, Save } from 'lucide-react'
import { useStore } from '../store'
import { api } from '../api/client'
import type { FlowNode } from '../types'

interface ConfigPanelProps {
  node: FlowNode
  pipelineId: string
  onClose: () => void
  onSave: (updated: FlowNode) => void
}

export function ConfigPanel({ node, pipelineId, onClose, onSave }: ConfigPanelProps) {
  const [label, setLabel] = useState(node.label)
  const [image, setImage] = useState(node.image)
  const [configText, setConfigText] = useState(
    JSON.stringify(node.config ?? {}, null, 2)
  )
  const [configError, setConfigError] = useState<string | null>(null)

  useEffect(() => {
    setLabel(node.label)
    setImage(node.image)
    setConfigText(JSON.stringify(node.config ?? {}, null, 2))
    setConfigError(null)
  }, [node.id])

  const handleSave = () => {
    let config: Record<string, unknown>
    try {
      config = JSON.parse(configText)
      setConfigError(null)
    } catch {
      setConfigError('Invalid JSON')
      return
    }
    onSave({ ...node, label, image, config })
  }

  return (
    <div className="h-full flex flex-col bg-panel">
      <div className="flex items-center justify-between p-3 border-b border-border">
        <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Node Config
        </h2>
        <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        <div>
          <label className="block text-xs text-gray-400 mb-1">Label</label>
          <input
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            className="w-full bg-canvas border border-border rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-accent"
          />
        </div>

        <div>
          <label className="block text-xs text-gray-400 mb-1">Docker Image</label>
          <input
            value={image}
            onChange={(e) => setImage(e.target.value)}
            placeholder="e.g. mirdain/uppercase:latest"
            className="w-full bg-canvas border border-border rounded px-2 py-1.5 text-sm text-white font-mono focus:outline-none focus:border-accent"
          />
        </div>

        <div>
          <label className="block text-xs text-gray-400 mb-1">Input Ports</label>
          <div className="space-y-1">
            {node.input_ports.map((p) => (
              <div key={p.id} className="flex items-center gap-2 text-xs text-blue-400">
                <span className="w-2 h-2 rounded-full bg-blue-500" />
                {p.label} <span className="text-gray-600">({p.id})</span>
              </div>
            ))}
            {node.input_ports.length === 0 && (
              <span className="text-xs text-gray-600">None (source node)</span>
            )}
          </div>
        </div>

        <div>
          <label className="block text-xs text-gray-400 mb-1">Output Ports</label>
          <div className="space-y-1">
            {node.output_ports.map((p) => (
              <div key={p.id} className="flex items-center gap-2 text-xs text-emerald-400">
                <span className="w-2 h-2 rounded-full bg-emerald-500" />
                {p.label} <span className="text-gray-600">({p.id})</span>
              </div>
            ))}
            {node.output_ports.length === 0 && (
              <span className="text-xs text-gray-600">None (sink node)</span>
            )}
          </div>
        </div>

        <div>
          <label className="block text-xs text-gray-400 mb-1">Config (JSON)</label>
          <textarea
            value={configText}
            onChange={(e) => setConfigText(e.target.value)}
            rows={6}
            className="w-full bg-canvas border border-border rounded px-2 py-1.5 text-xs text-white font-mono focus:outline-none focus:border-accent resize-none"
          />
          {configError && (
            <p className="text-xs text-red-400 mt-1">{configError}</p>
          )}
        </div>
      </div>

      <div className="p-3 border-t border-border">
        <button
          onClick={handleSave}
          className="w-full flex items-center justify-center gap-2 bg-accent hover:bg-accent-hover text-white text-sm py-1.5 rounded transition-colors"
        >
          <Save className="w-4 h-4" />
          Save Node
        </button>
      </div>
    </div>
  )
}
