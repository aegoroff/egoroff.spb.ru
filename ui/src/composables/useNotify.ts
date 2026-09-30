import axios from 'axios'
import { readonly, ref } from 'vue'

type NotificationKind = 'success' | 'danger'

interface Notification {
  id: number
  kind: NotificationKind
  text: string
}

const DISMISS_DELAY_MS = 5000
const MAX_DETAILS_LENGTH = 200

const notifications = ref<Array<Notification>>([])
const timers = new Map<number, ReturnType<typeof setTimeout>>()
let nextId = 1

const dismiss = (id: number): void => {
  clearTimeout(timers.get(id))
  timers.delete(id)
  notifications.value = notifications.value.filter((n) => n.id !== id)
}

const push = (kind: NotificationKind, text: string): void => {
  const id = nextId++
  notifications.value.push({ id, kind, text })
  timers.set(id, setTimeout(() => dismiss(id), DISMISS_DELAY_MS))
}

// Plain-text server responses carry useful details; HTML error pages do not.
function errorMessage(error: unknown): string {
  if (axios.isAxiosError(error)) {
    const data = error.response?.data
    if (typeof data === 'string' && data.trim() && !data.trimStart().startsWith('<')) {
      const text = data.trim()
      return text.length > MAX_DETAILS_LENGTH ? `${text.slice(0, MAX_DETAILS_LENGTH)}…` : text
    }
    return error.message
  }
  return error instanceof Error ? error.message : String(error)
}

export function useNotify() {
  return {
    notifications: readonly(notifications),
    dismiss,
    success: (text: string): void => push('success', text),
    error: (text: string, error?: unknown): void => {
      if (error !== undefined) {
        console.error(text, error)
      }
      push('danger', error === undefined ? text : `${text}: ${errorMessage(error)}`)
    },
  }
}
