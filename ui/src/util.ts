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
    const v = Reflect.get(q, key)
    if(!v) {
      continue
    }
    if (str !== '?') {
      str += '&'
    }
    str += key + '=' + encodeURIComponent(v)
  }
  return str
}

export function closeModalById(id: string): void {
  const modal = document.getElementById(id)
  if (!modal) return

  const dismissBtn = modal.querySelector(
    '[data-bs-dismiss="modal"]'
  ) as HTMLButtonElement | null

  dismissBtn?.click()
}