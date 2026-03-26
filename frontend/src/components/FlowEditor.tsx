import { useCallback, useRef, useState } from 'react'
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  addEdge,
  useNodesState,
  useEdgesState,
  type Connection,
  type Edge,
  type Node,
  type NodeTypes,
  BackgroundVariant,
  MarkerType,
  Panel,
} from 'reactflow'
import 'reactflow/dist/style.css'

import { ContainerNode } from './ContainerNode'
import { ConfigPanel } from './ConfigPanel'
import { NodeLibrary } from './NodeLibrary'
import { Toolbar } from './Toolbar'
import { LogPanel } from './LogPanel'
import { MetricsPanel } from './MetricsPanel'
import { useStore } from '../store'
import { api } from '../api/client'
import type { FlowNode, FlowEdge, NodeType, Pipeline } from '../types'
import { useQuery } from '@tanstack/react-query'

const rfNodeTypes: NodeTypes = { container: ContainerNode }

interface FlowEditorProps {
  pipeline: Pipeline
  onUpdate: (p: Pipeline) => void
}

function toRfNode(n: FlowNode): Node {
  return { id: n.id, type: 'container', position: n.position, data: n }
}

function toRfEdge(e: FlowEdge): Edge {
  return {
    id: e.id,
    source: e.source,
    sourceHandle: e.sourceHandle,
    target: e.target,
    targetHandle: e.targetHandle,
    animated: true,
    style: { stroke: '#6366f1', strokeWidth: 2 },
    markerEnd: { type: MarkerType.ArrowClosed, color: '#6366f1' },
  }
}

type BottomTab = 'logs' | 'metrics'

export function FlowEditor({ pipeline, onUpdate }: FlowEditorProps) {
  const reactFlowWrapper = useRef<HTMLDivElement>(null)
  const [rfInstance, setRfInstance] = useState<any>(null)
  const [bottomTab, setBottomTab] = useState<BottomTab>('logs')

  const [nodes, setNodes, onNodesChange] = useNodesState(pipeline.nodes.map(toRfNode))
  const [edges, setEdges, onEdgesChange] = useEdgesState(pipeline.edges.map(toRfEdge))

  const selectedNodeId = useStore((s) => s.selectedNodeId)
  const setSelectedNodeId = useStore((s) => s.setSelectedNodeId)
  const selectedNode = nodes.find((n) => n.id === selectedNodeId)?.data as FlowNode | undefined

  // Fetch node types for ConfigPanel lookup
  const { data: nodeTypes = [] } = useQuery({
    queryKey: ['node-types'],
    queryFn: api.listNodeTypes,
  })

  const buildPipeline = useCallback(
    (ns: Node[], es: Edge[]): Pipeline => ({
      ...pipeline,
      nodes: ns.map((n) => ({ ...n.data, position: n.position })),
      edges: es.map((e) => ({
        id: e.id,
        source: e.source,
        sourceHandle: e.sourceHandle ?? 'output',
        target: e.target,
        targetHandle: e.targetHandle ?? 'input',
      })),
    }),
    [pipeline]
  )

  const save = useCallback(
    async (ns: Node[], es: Edge[]) => {
      const updated = buildPipeline(ns, es)
      try {
        const saved = await api.updatePipeline(pipeline.id, {
          name: updated.name,
          nodes: updated.nodes,
          edges: updated.edges,
        })
        onUpdate(saved)
      } catch (err) {
        console.error('Save failed', err)
      }
    },
    [buildPipeline, pipeline.id, onUpdate]
  )

  const onConnect = useCallback(
    (params: Connection) => {
      const newEdge: Edge = {
        ...params,
        id: `e-${params.source}-${params.sourceHandle}-${params.target}-${params.targetHandle}`,
        animated: true,
        style: { stroke: '#6366f1', strokeWidth: 2 },
        markerEnd: { type: MarkerType.ArrowClosed, color: '#6366f1' },
      } as Edge
      setEdges((eds) => {
        const updated = addEdge(newEdge, eds)
        save(nodes, updated)
        return updated
      })
    },
    [nodes, save, setEdges]
  )

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault()
      const raw = event.dataTransfer.getData('application/mirdain-nodetype')
      if (!raw) return
      const nodeType: NodeType = JSON.parse(raw)

      const bounds = reactFlowWrapper.current!.getBoundingClientRect()
      const position = rfInstance.screenToFlowPosition({
        x: event.clientX - bounds.left,
        y: event.clientY - bounds.top,
      })

      const id = `node-${Date.now()}`
      const newNode: Node = {
        id,
        type: 'container',
        position,
        data: {
          id,
          label: nodeType.name,
          image: nodeType.image,
          position,
          config: nodeType.default_config ?? {},
          input_ports: nodeType.input_ports,
          output_ports: nodeType.output_ports,
        } satisfies FlowNode,
      }
      setNodes((nds) => {
        const updated = [...nds, newNode]
        save(updated, edges)
        return updated
      })
    },
    [rfInstance, edges, save, setNodes]
  )

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    setSelectedNodeId(node.id)
  }, [setSelectedNodeId])

  const onPaneClick = useCallback(() => setSelectedNodeId(null), [setSelectedNodeId])

  const handleNodeSave = useCallback(
    (updated: FlowNode) => {
      setNodes((nds) => {
        const next = nds.map((n) => n.id === updated.id ? { ...n, data: updated } : n)
        save(next, edges)
        return next
      })
    },
    [edges, save, setNodes]
  )

  // Find the NodeType for the selected node (to pass to ConfigPanel)
  const selectedNodeType = selectedNode
    ? nodeTypes.find((nt) => nt.image === selectedNode.image)
    : undefined

  return (
    <div className="flex flex-col h-full">
      <Toolbar pipeline={pipeline} onUpdate={onUpdate} />

      <div className="flex flex-1 overflow-hidden">
        {/* Left: node library */}
        <div className="w-52 flex-shrink-0 border-r border-border bg-panel overflow-hidden">
          <NodeLibrary
            onDragStart={(e, nt) => {
              e.dataTransfer.setData('application/mirdain-nodetype', JSON.stringify(nt))
              e.dataTransfer.effectAllowed = 'move'
            }}
          />
        </div>

        {/* Centre: canvas + bottom panel */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <div ref={reactFlowWrapper} className="flex-1">
            <ReactFlow
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              onDrop={onDrop}
              onDragOver={onDragOver}
              onNodeClick={onNodeClick}
              onPaneClick={onPaneClick}
              onInit={setRfInstance}
              nodeTypes={rfNodeTypes}
              fitView
              deleteKeyCode="Delete"
              className="bg-canvas"
            >
              <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#2a2d3a" />
              <Controls className="!bg-panel !border-border" />
              <MiniMap className="!bg-panel !border-border" nodeColor="#6366f1" maskColor="rgba(15,17,23,0.8)" />
              <Panel position="top-left" className="text-xs text-gray-600 ml-2 mt-2">
                {nodes.length} nodes · {edges.length} edges
              </Panel>
            </ReactFlow>
          </div>

          {/* Bottom tabbed panel */}
          <div className="h-40 border-t border-border bg-panel flex flex-col overflow-hidden">
            <div className="flex border-b border-border flex-shrink-0">
              <BottomTabBtn active={bottomTab === 'logs'} onClick={() => setBottomTab('logs')}>
                Logs
              </BottomTabBtn>
              <BottomTabBtn active={bottomTab === 'metrics'} onClick={() => setBottomTab('metrics')}>
                Metrics
              </BottomTabBtn>
            </div>
            <div className="flex-1 overflow-hidden">
              {bottomTab === 'logs' && <LogPanel pipelineId={pipeline.id} />}
              {bottomTab === 'metrics' && (
                <MetricsPanel nodes={pipeline.nodes} />
              )}
            </div>
          </div>
        </div>

        {/* Right: config panel */}
        {selectedNode && (
          <div className="w-64 flex-shrink-0 border-l border-border overflow-hidden">
            <ConfigPanel
              node={selectedNode}
              pipelineId={pipeline.id}
              nodeType={selectedNodeType}
              onClose={() => setSelectedNodeId(null)}
              onSave={handleNodeSave}
            />
          </div>
        )}
      </div>
    </div>
  )
}

function BottomTabBtn({
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
      className={`text-xs px-4 py-1.5 transition-colors border-b-2 ${
        active
          ? 'text-white border-accent'
          : 'text-gray-500 hover:text-gray-300 border-transparent'
      }`}
    >
      {children}
    </button>
  )
}
