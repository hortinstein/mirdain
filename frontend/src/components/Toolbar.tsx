import { useState } from 'react'
import { Play, Square, Zap, Save, Loader2 } from 'lucide-react'
import { api } from '../api/client'
import { useStore } from '../store'
import type { Pipeline } from '../types'

interface ToolbarProps {
  pipeline: Pipeline
  onUpdate: (p: Pipeline) => void
}

export function Toolbar({ pipeline, onUpdate }: ToolbarProps) {
  const [deploying, setDeploying] = useState(false)
  const [stopping, setStopping] = useState(false)
  const [triggering, setTriggering] = useState(false)
  const [showTrigger, setShowTrigger] = useState(false)

  const triggerInput = useStore((s) => s.triggerInput)
  const setTriggerInput = useStore((s) => s.setTriggerInput)

  const isRunning = pipeline.status === 'running'

  const handleDeploy = async () => {
    setDeploying(true)
    try {
      await api.deployPipeline(pipeline.id)
      const updated = await api.getPipeline(pipeline.id)
      onUpdate(updated)
    } catch (e) {
      alert(`Deploy failed: ${e}`)
    } finally {
      setDeploying(false)
    }
  }

  const handleStop = async () => {
    setStopping(true)
    try {
      await api.stopPipeline(pipeline.id)
      const updated = await api.getPipeline(pipeline.id)
      onUpdate(updated)
    } catch (e) {
      alert(`Stop failed: ${e}`)
    } finally {
      setStopping(false)
    }
  }

  const handleTrigger = async () => {
    let data: unknown
    try {
      data = JSON.parse(triggerInput)
    } catch {
      alert('Invalid JSON in trigger input')
      return
    }
    setTriggering(true)
    try {
      const result = await api.triggerPipeline(pipeline.id, data)
      console.log('Trigger result:', result)
      setShowTrigger(false)
    } catch (e) {
      alert(`Trigger failed: ${e}`)
    } finally {
      setTriggering(false)
    }
  }

  return (
    <div className="flex items-center gap-2 px-4 py-2 border-b border-border bg-panel flex-shrink-0">
      {/* Pipeline name */}
      <span className="text-sm font-semibold text-white mr-2">{pipeline.name}</span>

      {/* Status badge */}
      <span
        className={`text-xs px-2 py-0.5 rounded-full font-medium ${
          isRunning
            ? 'bg-green-900/50 text-green-400 border border-green-700'
            : 'bg-gray-800 text-gray-400 border border-gray-700'
        }`}
      >
        {pipeline.status}
      </span>

      <div className="flex-1" />

      {/* Actions */}
      {!isRunning ? (
        <button
          onClick={handleDeploy}
          disabled={deploying}
          className="flex items-center gap-1.5 bg-green-700 hover:bg-green-600 disabled:opacity-50 text-white text-xs px-3 py-1.5 rounded transition-colors"
        >
          {deploying ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5" />}
          Deploy
        </button>
      ) : (
        <button
          onClick={handleStop}
          disabled={stopping}
          className="flex items-center gap-1.5 bg-red-700 hover:bg-red-600 disabled:opacity-50 text-white text-xs px-3 py-1.5 rounded transition-colors"
        >
          {stopping ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Square className="w-3.5 h-3.5" />}
          Stop
        </button>
      )}

      <button
        onClick={() => setShowTrigger(true)}
        disabled={!isRunning}
        className="flex items-center gap-1.5 bg-accent hover:bg-accent-hover disabled:opacity-40 text-white text-xs px-3 py-1.5 rounded transition-colors"
      >
        <Zap className="w-3.5 h-3.5" />
        Trigger
      </button>

      {/* Trigger modal */}
      {showTrigger && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
          <div className="bg-panel border border-border rounded-xl p-5 w-96 shadow-2xl">
            <h3 className="text-sm font-semibold text-white mb-3">Trigger Pipeline</h3>
            <p className="text-xs text-gray-400 mb-2">Input JSON data to send to source nodes:</p>
            <textarea
              value={triggerInput}
              onChange={(e) => setTriggerInput(e.target.value)}
              rows={6}
              className="w-full bg-canvas border border-border rounded px-2 py-1.5 text-xs text-white font-mono focus:outline-none focus:border-accent resize-none"
            />
            <div className="flex gap-2 mt-3 justify-end">
              <button
                onClick={() => setShowTrigger(false)}
                className="text-xs text-gray-400 hover:text-white px-3 py-1.5 rounded border border-border"
              >
                Cancel
              </button>
              <button
                onClick={handleTrigger}
                disabled={triggering}
                className="flex items-center gap-1.5 bg-accent hover:bg-accent-hover text-white text-xs px-3 py-1.5 rounded"
              >
                {triggering ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Zap className="w-3.5 h-3.5" />}
                Send
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
