import { authFetch } from './client'
import type { SseEventHandler } from './types'

export function streamChat(sessionId: string, handlers: SseEventHandler): AbortController {
  const controller = new AbortController()
  authFetch(`/chat/stream/${encodeURIComponent(sessionId)}`, { signal: controller.signal })
    .then(async (response) => {
      const reader = response.body?.getReader()
      if (!reader) return
      const decoder = new TextDecoder()
      let buffer = ''
      let currentEvent = ''
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() || ''
        for (const line of lines) {
          const trimmed = line.trim()
          if (trimmed.startsWith('event: ')) { currentEvent = trimmed.slice(7).trim(); continue }
          if (!trimmed.startsWith('data: ')) continue
          const data = trimmed.slice(6)
          switch (currentEvent) {
            case 'token': handlers.onToken?.(data); break
            case 'reasoning': handlers.onReasoning?.(data); break
            case 'status': handlers.onStatus?.(data); break
            case 'error': handlers.onError?.(data); break
            case 'new_round': handlers.onNewRound?.(); break
            case 'tool_executed': try { handlers.onToolExecuted?.(JSON.parse(data)) } catch { /* ignore */ } break
            case 'image_generated': try { handlers.onImageGenerated?.(JSON.parse(data)) } catch { /* ignore */ } break
            case 'done': try { const p = JSON.parse(data); handlers.onDone?.(p.usage, p.quality) } catch { /* ignore */ } break
            case 'evaluation': try { handlers.onEvaluation?.(JSON.parse(data)) } catch { /* ignore */ } break
            case 'quality_score': try { handlers.onQualityScore?.(JSON.parse(data)) } catch { /* ignore */ } break
          }
        }
      }
    })
    .catch((err) => { if (err.name !== 'AbortError') handlers.onError?.(String(err)) })
  return controller
}
