export interface SessionMeta {
  id: string
  title: string
  message_count: number
  created_at: number
  agent_id: string
}

export interface ToolSchema {
  type: string
  function: {
    name: string
    description: string
    parameters: Record<string, unknown>
  }
}

export interface PluginInfo {
  name: string
  version: string
  description: string
  author: string | null
  enabled: boolean
}

// ── Skills ──

export interface SkillInfo {
  name: string
  description: string
  parameters: Record<string, unknown> | null
  content: string
}

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

// ── Stats ──

export interface StatsResponse {
  total_requests: number
  total_tokens: number
  total_cost_usd: number
  today: {
    requests: number
    tokens: number
    cost_usd: number
  }
}

export interface AgentInfo {
  id: string
  provider: string
  model: string
  base_url: string
  tool_count: number
  enabled_tools: string[]
  system_prompt: string | null
  is_sub_agent?: boolean
  provider_ref?: string | null
}

// ── Sessions ──

export interface CurrentSession {
  id: string | null
  title: string | null
  message_count: number
  agent_id?: string | null
  messages: { role: string; content?: string; reasoning?: string; name?: string; args?: string; result?: string }[]
}

// ── Tool call event ──

export interface ToolCallEvent {
  name: string
  args: string
  result: string
  step: number
  total_steps: number
}

// ── Image generated event ──

export interface ImageGeneratedEvent {
  path: string
  alt_text: string
  format: string
  width: number
  height: number
}

export interface EvaluationEvent {
  tool: string
  valid: boolean
  issues: string[]
}

export interface QualityScore {
  score: string
  complete: boolean
  issues: string[]
  references_valid: boolean
}

// ── SSE Chat Stream ──

export type SseEventHandler = {
  onToken?: (token: string) => void
  onReasoning?: (text: string) => void
  onStatus?: (text: string) => void
  onError?: (error: string) => void
  onDone?: (usage: TokenUsage | null, quality?: QualityScore | null) => void
  onNewRound?: () => void
  onToolExecuted?: (evt: ToolCallEvent) => void
  onImageGenerated?: (evt: ImageGeneratedEvent) => void
  onEvaluation?: (evt: EvaluationEvent) => void
  onQualityScore?: (evt: QualityScore) => void
}

// ── Token usage ──

export interface TokenUsage {
  prompt_tokens?: number
  completion_tokens?: number
  total_tokens?: number
  estimated_cost_usd?: number
}

// ── Chat message type ──

export interface ToolCallMsg {
  name: string
  args: string
  result: string
  step: number
  total_steps: number
}

export type ChatMessage = {
  role: 'user' | 'assistant' | 'error' | 'image' | 'evaluation' | 'quality' | 'feedback'
  content: string
  reasoning?: string
  toolCalls?: ToolCallMsg[]
  tokenUsage?: TokenUsage
  image?: {
    path: string
    alt_text: string
    width: number
    height: number
    format: string
    url: string
  }
  evaluation?: EvaluationEvent
  quality?: QualityScore
  feedback?: FeedbackEvent
}

export interface FeedbackEvent {
  positive: boolean
  message?: string
}
