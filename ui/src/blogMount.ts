import { App, createApp } from 'vue'
import dayjs from 'dayjs'
import 'dayjs/locale/ru'
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'

dayjs.locale('ru')

let blogAnnouncesApp: App | null = null
let blogTitleApp: App | null = null

function blogFilterTitle(query: string): string {
  const params = new URLSearchParams(query)
  const tag = params.get('tag')
  if (tag) {
    return `все посты по метке: ${tag}`
  }
  const year = params.get('year')
  const month = params.get('month')
  if (year && month) {
    const m = dayjs(new Date(Number(year), Number(month) - 1, 1))
    return `записи за ${m.format('MMMM YYYY')}`
  }
  if (year) {
    return `записи за ${year} год`
  }
  return 'тут я пишу'
}

export function remountBlogFilter(query: string, title: string): void {
  blogAnnouncesApp?.unmount()
  blogTitleApp?.unmount()

  blogAnnouncesApp = createApp(BlogAnnounces, { q: query })
  blogAnnouncesApp.mount('#blogcontainer')

  blogTitleApp = createApp(BlogTitle, { text: title })
  blogTitleApp.mount('#blogSmallTitle')
}

export function remountBlogFilterFromHash(hash: string): void {
  remountBlogFilter(hash, blogFilterTitle(hash))
}
