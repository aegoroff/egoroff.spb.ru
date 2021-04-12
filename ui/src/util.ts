// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import { Query } from '@/services/ApiService.vue'
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import { SearchQuery } from '@/services/SearchService.vue'

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
    const v = Reflect.get(q, key)
    str += key + '=' + encodeURIComponent(v)
  }
  return str
}
