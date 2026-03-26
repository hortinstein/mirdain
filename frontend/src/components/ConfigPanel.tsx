import { useState, useEffect } from 'react'
import { X, Save, Sparkles, RefreshCw, Loader2, Code2 } from 'lucide-react'
import { useQueryClient } from '@tanstack/react-query'
import { api } from '../api/client'
import type { FlowNode, NodeType } from '../types'

interface ConfigPanelProps {
  node: FlowNode
  pipelineId: string
  nodeType?: NodeType
  onClose: () => void
  onSave: (updated: FlowNode) => void
}

type Tab = 'config' | 'code'

export function ConfigPanel({ node, pipelineId, nodeType, onClose, onSave }: ConfigPanelProps) {
  const queryClient = useQueryClient()
  const [tab, setTab] = useState<Tab>('config')
  const [label, setLabel] = useState(node.label)
  const [image, setImage] = useState(node.image)
  const [configText, setConfigText] = useState(JSON.stringify(node.config ?? {}, null, 2))
  const [configError, setConfigError] = useState<string | null>(null)

  // Code tab
  const [sourceCode, setSourceCode] = useState(nodeType?.source_code ?? '')
  const [description, setDescription] = useState(nodeType?.ai_prompt ?? '')
  const [rebuilding, setRebuilding] = useState(false)
  const [rebuildMsg, setRebuildMsg] = useState<string | null>(null)

  const isAiNode = !!nodeType?.ai_prompt

  useEffect(() => {
    setLabel(node.label)
    setImage(node.image)
    setConfigText(JSON.stringify(node.config ?? {}, null, 2))
    setConfigError(null)
    setSourceCode(nodeType?.source_code ?? '')
    setDescription(nodeType?.ai_prompt ?? '')
    setRebuildMsg(null)
  }, [node.id, nodeType?.id])

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

  const handleRebuild = async (regenerate: boolean) => {
    if (!nodeType) return
    setRebuilding(true)
    setRebuildMsg(null)
    try {
      await api.rebuildNodeType(nodeType.id, {
        description: regenerate ? description : undefined,
        source_code: regenerate ? undefined : sourceCode,
      })
      queryClient.invalidateQueries({ queryKey: ['node-types'] })
      setRebuildMsg(regenerate ? 'Regenerating and rebuilding...' : 'Rebuilding with updated code...')
    } catch (e: any) {
      setRebuildMsg(`Error: ${e?.message ?? e}`)
    } finally {
      setRebuilding(false)
    }
  }

  return (
    <div className="h-full flex flex-col bg-panel">
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b border-border">
        <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Node Config
        </h2>
        <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Tabs (only when AI node) */}
      {isAiNode && (
        <div className="flex border-b border-border">
          <TabBtn active={tab === 'config'} onClick={() => setTab('config')}>Config</TabBtn>
          <TabBtn active={tab === 'code'} onClick={() => setTab('code')}>
            <Code2 className="w-3 h-3 mr-1" />
            Source
          </TabBtn>
        </div>
      )}

      {/* Config tab */}
      {tab === 'config' && (
        <div className="flex-1 overflow-y-auto p-3 space-y-4">
          <Field label="Label">
            <input
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              className="input-base"
            />
          </Field>

          <Field label="Docker Image">
            <input
              value={image}
              onChange={(e) => setImage(e.target.value)}
              placeholder="e.g. mirdain/uppercase:latest"
              className="input-base font-mono"
            />
          </Field>

          <Field label="Input Ports">
            <div className="space-y-1">
              {node.input_ports.map((p) => (
                <div key={p.id} className="flex items-center gap-2 text-xs text-blue-400">
                  <span className="w-2 h-2 rounded-full bg-blue-500" />
                  {p.label} <span className="text-gray-600">({p.id})</span>
                </div>
              ))}
              {node.input_ports.length === 0 && <span className="text-xs text-gray-600">None</span>}
            </div>
          </Field>

          <Field label="Output Ports">
            <div className="space-y-1">
              {node.output_ports.map((p) => (
                <div key={p.id} className="flex items-center gap-2 text-xs text-emerald-400">
                  <span className="w-2 h-2 rounded-full bg-emerald-500" />
                  {p.label} <span className="text-gray-600">({p.id})</span>
                </div>
              ))}
              {node.output_ports.length === 0 && <span className="text-xs text-gray-600">None</span>}
            </div>
          </Field>

          <Field label="Config (JSON)">
            <textarea
              value={configText}
              onChange={(e) => setConfigText(e.target.value)}
              rows={6}
              className="input-base font-mono text-xs resize-none"
            />
            {configError && <p className="text-xs text-red-400 mt-1">{configError}</p>}
          </Field>
        </div>
      )}

      {/* Code tab */}
      {tab === 'code' && isAiNode && (
        <div className="flex-1 overflow-y-auto p-3 space-y-4">
          <Field label="AI Description / Prompt">
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              className="input-base text-xs resize-none"
              placeholder="Describe what this processor should do..."
            />
            <button
              onClick={() => handleRebuild(true)}
              disabled={rebuilding}
              className="mt-2 w-full flex items-center justify-center gap-1.5 bg-purple-700 hover:bg-purple-600 disabled:opacity-40 text-white text-xs py-1.5 rounded transition-colors"
            >
              {rebuilding ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Sparkles className="w-3.5 h-3.5" />}
              Regenerate with AI
            </button>
          </Field>

          <Field label="Rust Source Code">
            <textarea
              value={sourceCode}
              onChange={(e) => setSourceCode(e.target.value)}
              rows={16}
              className="input-base font-mono text-[10px] leading-relaxed resize-none"
              spellCheck={false}
            />
            <button
              onClick={() => handleRebuild(false)}
              disabled={rebuilding}
              className="mt-2 w-full flex items-center justify-center gap-1.5 bg-blue-700 hover:bg-blue-600 disabled:opacity-40 text-white text-xs py-1.5 rounded transition-colors"
            >
              {rebuilding ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <RefreshCw className="w-3.5 h-3.5" />}
              Rebuild Container
            </button>
          </Field>

          {rebuildMsg && (
            <p className="text-xs text-yellow-400">{rebuildMsg}</p>
          )}
        </div>
      )}

      {/* Footer (only config tab) */}
      {tab === 'config' && (
        <div className="p-3 border-t border-border">
          <button
            onClick={handleSave}
            className="w-full flex items-center justify-center gap-2 bg-accent hover:bg-accent-hover text-white text-sm py-1.5 rounded transition-colors"
          >
            <Save className="w-4 h-4" />
            Save Node
          </button>
        </div>
      )}
    </div>
  )
}

function TabBtn({
  active,
  onClick,
  children,
}: {
  active: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      onClick={onClick}
      className={`flex-1 flex items-center justify-center text-xs py-2 transition-colors ${
        active
          ? 'text-white border-b-2 border-accent'
          : 'text-gray-500 hover:text-gray-300 border-b-2 border-transparent'
      }`}
    >
      {children}
    </button>
  )
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <label className="block text-xs text-gray-400 mb-1">{label}</label>
      {children}
    </div>
  )
}
