import { SearchQuery } from '@/services/SearchService'
import type { Query } from './models/blog'

const RU = 'ru-RU'

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

export function formatLongDate(date: Date | string): string {
  return new Date(date).toLocaleDateString(RU, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
}

export function formatMonthName(month: number): string {
  const name = new Intl.DateTimeFormat(RU, { month: 'long' }).format(
    new Date(2000, month - 1, 1)
  )
  return name.charAt(0).toUpperCase() + name.slice(1)
}

export function formatFromNow(date: Date | string): string {
  const rtf = new Intl.RelativeTimeFormat('ru', { numeric: 'auto' })
  const delta = (new Date(date).getTime() - Date.now()) / 1000
  const divisions: [Intl.RelativeTimeFormatUnit, number][] = [
    ['year', 60 * 60 * 24 * 365],
    ['month', 60 * 60 * 24 * 30],
    ['day', 60 * 60 * 24],
    ['hour', 60 * 60],
    ['minute', 60],
    ['second', 1],
  ]
  for (const [unit, seconds] of divisions) {
    if (Math.abs(delta) >= seconds || unit === 'second') {
      return rtf.format(Math.round(delta / seconds), unit)
    }
  }
  return rtf.format(0, 'second')
}
