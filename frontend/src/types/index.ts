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

export type BuildStatus = 'ready' | 'building' | 'error'

export interface NodeType {
  id: string
  name: string
  image: string
  description: string
  input_ports: Port[]
  output_ports: Port[]
  default_config: Record<string, unknown>
  ai_prompt?: string
  source_code?: string
  build_status: BuildStatus
}

export interface NodeMetrics {
  messages_in: number
  bytes_in: number
  bytes_out: number
  total_latency_ms: number
  avg_latency_ms: number
  errors: number
}

export interface WsEvent {
  type:
    | 'pipeline_status'
    | 'node_status'
    | 'execution_log'
    | 'node_output'
    | 'metrics_update'
    | 'build_log'
    | 'build_complete'
    | 'build_error'
    | 'node_types_updated'
  // pipeline events
  pipelineId?: string
  status?: string
  message?: string
  data?: unknown
  // node events
  nodeId?: string
  // metrics
  metrics?: NodeMetrics
  // build events
  nodeTypeId?: string
  image?: string
}
