import { Query } from '@/services/ApiService'
import { SearchQuery } from '@/services/SearchService'

export function removeHash (): void {
  history.pushState('', document.title, window.location.pathname + window.location.search)
}

export function toQuery (q?: Query | SearchQuery): string {
  if (q === undefined) {
    return ''
  }
  let str = '?'
  for (const key in q) {
    if (str !== '?') {
      str += '&'
    }
    const v = (q as any)[key]
    if (v !== undefined && v !== null) {
      str += key + '=' + encodeURIComponent(v)
    }
  }
  return str === '?' ? '' : str
}

export function closeModalById(id: string): void {
  const modal = document.getElementById(id)
  if (!modal) return

  const dismissBtn = modal.querySelector(
    '[data-bs-dismiss="modal"]'
  ) as HTMLButtonElement | null

  dismissBtn?.click()
}