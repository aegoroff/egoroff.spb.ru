import { App, createApp } from 'vue'
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import { formatMonthYear } from '@/util'

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
    return `записи за ${formatMonthYear(Number(year), Number(month))}`
  }
  if (year) {
    return `записи за ${year} год`
  }
  return 'тут я пишу'
}

function prepareBlogPager(): void {
  let pager = document.getElementById('blogPager')
  if (!pager) {
    const container = document.getElementById('blogcontainer')
    if (!container) {
      return
    }
    pager = document.createElement('div')
    pager.id = 'blogPager'
    container.insertAdjacentElement('afterend', pager)
    return
  }
  if (pager.tagName === 'UL') {
    const div = document.createElement('div')
    div.id = 'blogPager'
    pager.replaceWith(div)
  }
}

export function remountBlogFilter(query: string, title: string): void {
  blogAnnouncesApp?.unmount()
  blogTitleApp?.unmount()

  prepareBlogPager()

  blogAnnouncesApp = createApp(BlogAnnounces, { q: query })
  blogAnnouncesApp.mount('#blogcontainer')

  blogTitleApp = createApp(BlogTitle, { text: title })
  blogTitleApp.mount('#blogSmallTitle')
}

export function remountBlogFilterFromHash(hash: string): void {
  remountBlogFilter(hash, blogFilterTitle(hash))
}
