export interface Port {
  id: string
  label: string
}

export interface FlowNode {
  id: string
  label: string
  image: string
  position: { x: number; y: number }
  config: Record<string, unknown>
  input_ports: Port[]
  output_ports: Port[]
}

export interface FlowEdge {
  id: string
  source: string
  sourceHandle: string
  target: string
  targetHandle: string
}

export type PipelineStatus = 'stopped' | 'running' | 'error'

export interface Pipeline {
  id: string
  name: string
  nodes: FlowNode[]
  edges: FlowEdge[]
  status: PipelineStatus
}

export interface NodeType {
  id: string
  name: string
  image: string
  description: string
  input_ports: Port[]
  output_ports: Port[]
  default_config: Record<string, unknown>
}

export interface WsEvent {
  type: 'pipeline_status' | 'node_status' | 'execution_log' | 'node_output'
  pipelineId?: string
  nodeId?: string
  status?: string
  message?: string
  data?: unknown
}
