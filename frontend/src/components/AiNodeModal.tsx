import { useState } from 'react'
import { X, Plus, Trash2, Sparkles, Loader2, CheckCircle2, AlertCircle } from 'lucide-react'
import { useQueryClient } from '@tanstack/react-query'
import { api } from '../api/client'
import { useStore } from '../store'

interface PortDef {
  id: string
  label: string
}

export function AiNodeModal({ onClose }: { onClose: () => void }) {
  const queryClient = useQueryClient()
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [inputs, setInputs] = useState<PortDef[]>([{ id: 'input', label: 'Input' }])
  const [outputs, setOutputs] = useState<PortDef[]>([{ id: 'output', label: 'Output' }])
  const [state, setState] = useState<'idle' | 'generating' | 'building' | 'done' | 'error'>('idle')
  const [error, setError] = useState<string | null>(null)
  const [generatedId, setGeneratedId] = useState<string | null>(null)

  const buildStatuses = useStore((s) => s.buildStatuses)
  const logs = useStore((s) => s.logs.filter((l) => l.kind === 'build' && l.nodeTypeId === generatedId))

  // Once building starts, watch build status
  const buildStatus = generatedId ? buildStatuses[generatedId] : undefined
  if (state === 'building' && buildStatus === 'ready') setState('done')
  if (state === 'building' && buildStatus === 'error') setState('error')

  const addPort = (setter: typeof setInputs) =>
    setter((prev) => [...prev, { id: `port${prev.length + 1}`, label: `Port ${prev.length + 1}` }])

  const removePort = (setter: typeof setInputs, idx: number) =>
    setter((prev) => prev.filter((_, i) => i !== idx))

  const updatePort = (setter: typeof setInputs, idx: number, field: 'id' | 'label', val: string) =>
    setter((prev) => prev.map((p, i) => (i === idx ? { ...p, [field]: val } : p)))

  const handleGenerate = async () => {
    if (!name.trim() || !description.trim()) return
    setState('generating')
    setError(null)
    try {
      const { node_type } = await api.generateNodeType({
        name: name.trim(),
        description: description.trim(),
        input_ports: inputs,
        output_ports: outputs,
      })
      setGeneratedId(node_type.id)
      setState('building')
      queryClient.invalidateQueries({ queryKey: ['node-types'] })
    } catch (e: any) {
      setError(String(e?.message ?? e))
      setState('error')
    }
  }

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-panel border border-border rounded-2xl w-full max-w-2xl shadow-2xl flex flex-col max-h-[90vh]">
        {/* Header */}
        <div className="flex items-center justify-between p-5 border-b border-border">
          <div className="flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-accent" />
            <h2 className="text-base font-semibold text-white">Generate Node with AI</h2>
          </div>
          <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-5 space-y-5">
          {/* Name */}
          <div>
            <label className="block text-xs text-gray-400 mb-1.5">Node Name</label>
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. JSON Formatter"
              className="w-full bg-canvas border border-border rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-accent"
            />
          </div>

          {/* Description */}
          <div>
            <label className="block text-xs text-gray-400 mb-1.5">
              Description <span className="text-gray-600">(what should this processor do?)</span>
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="e.g. Parse a JSON string from the input field and pretty-print it with 2-space indentation. Return an error message if parsing fails."
              rows={4}
              className="w-full bg-canvas border border-border rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-accent resize-none"
            />
          </div>

          {/* Ports */}
          <div className="grid grid-cols-2 gap-4">
            <PortList
              label="Input Ports"
              ports={inputs}
              color="blue"
              onAdd={() => addPort(setInputs)}
              onRemove={(i) => removePort(setInputs, i)}
              onUpdate={(i, f, v) => updatePort(setInputs, i, f, v)}
            />
            <PortList
              label="Output Ports"
              ports={outputs}
              color="emerald"
              onAdd={() => addPort(setOutputs)}
              onRemove={(i) => removePort(setOutputs, i)}
              onUpdate={(i, f, v) => updatePort(setOutputs, i, f, v)}
            />
          </div>

          {/* Build log */}
          {(state === 'building' || state === 'done' || (state === 'error' && logs.length > 0)) && (
            <div>
              <div className="flex items-center gap-2 mb-1.5">
                <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                  Build Log
                </span>
                {state === 'done' && <CheckCircle2 className="w-3.5 h-3.5 text-green-400" />}
                {state === 'error' && <AlertCircle className="w-3.5 h-3.5 text-red-400" />}
                {state === 'building' && <Loader2 className="w-3.5 h-3.5 text-yellow-400 animate-spin" />}
              </div>
              <div className="bg-canvas rounded-lg border border-border p-3 h-40 overflow-y-auto font-mono text-xs space-y-0.5">
                {logs.map((l) => (
                  <div key={l.id} className="text-gray-400">{l.message}</div>
                ))}
                {logs.length === 0 && (
                  <span className="text-gray-600">Waiting for build output...</span>
                )}
              </div>
            </div>
          )}

          {/* Error */}
          {state === 'error' && error && (
            <div className="bg-red-900/20 border border-red-700 rounded-lg p-3 text-sm text-red-400">
              {error}
            </div>
          )}

          {/* Success */}
          {state === 'done' && (
            <div className="bg-green-900/20 border border-green-700 rounded-lg p-3 text-sm text-green-400 flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 flex-shrink-0" />
              Node type built successfully! Find it in the Node Library.
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-5 border-t border-border flex justify-end gap-3">
          <button
            onClick={onClose}
            className="text-sm text-gray-400 hover:text-white px-4 py-2 rounded-lg border border-border transition-colors"
          >
            {state === 'done' ? 'Close' : 'Cancel'}
          </button>
          {state !== 'done' && (
            <button
              onClick={handleGenerate}
              disabled={!name.trim() || !description.trim() || state === 'generating' || state === 'building'}
              className="flex items-center gap-2 bg-accent hover:bg-accent-hover disabled:opacity-40 text-white text-sm px-4 py-2 rounded-lg transition-colors"
            >
              {(state === 'generating' || state === 'building') ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Sparkles className="w-4 h-4" />
              )}
              {state === 'generating' ? 'Generating...' : state === 'building' ? 'Building...' : 'Generate'}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

function PortList({
  label,
  ports,
  color,
  onAdd,
  onRemove,
  onUpdate,
}: {
  label: string
  ports: PortDef[]
  color: 'blue' | 'emerald'
  onAdd: () => void
  onRemove: (i: number) => void
  onUpdate: (i: number, field: 'id' | 'label', val: string) => void
}) {
  const accent = color === 'blue' ? 'text-blue-400' : 'text-emerald-400'
  return (
    <div>
      <div className="flex items-center justify-between mb-1.5">
        <label className={`text-xs font-medium ${accent}`}>{label}</label>
        <button
          onClick={onAdd}
          className="text-gray-500 hover:text-white transition-colors"
          title={`Add ${label}`}
        >
          <Plus className="w-3.5 h-3.5" />
        </button>
      </div>
      <div className="space-y-1.5">
        {ports.map((p, i) => (
          <div key={i} className="flex gap-1.5">
            <input
              value={p.id}
              onChange={(e) => onUpdate(i, 'id', e.target.value)}
              placeholder="id"
              className="w-1/2 bg-canvas border border-border rounded px-2 py-1 text-xs text-white font-mono focus:outline-none focus:border-accent"
            />
            <input
              value={p.label}
              onChange={(e) => onUpdate(i, 'label', e.target.value)}
              placeholder="label"
              className="flex-1 bg-canvas border border-border rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-accent"
            />
            {ports.length > 1 && (
              <button
                onClick={() => onRemove(i)}
                className="text-gray-600 hover:text-red-400 transition-colors flex-shrink-0"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
