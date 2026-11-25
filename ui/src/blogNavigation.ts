export function enableBlogNavigation(
  results: HTMLElement,
  decorate: (root: ParentNode) => () => void,
  onNavigate: (url: URL, page: Document) => void,
): void {
  const blog = results.closest('[data-blog-list]')
  if (!blog) return

  let disposeResults = decorate(results)
  let displayedUrl = window.location.href
  let pending: AbortController | undefined

  async function navigate(url: URL, fromHistory: boolean): Promise<void> {
    pending?.abort()
    if (url.href === displayedUrl && url.href === window.location.href) {
      results.removeAttribute('aria-busy')
      return
    }

    const request = new AbortController()
    pending = request
    results.setAttribute('aria-busy', 'true')

    try {
      const response = await fetch(url, { signal: request.signal })
      if (!response.ok) throw new Error(`Blog navigation returned ${response.status}`)

      const page = new DOMParser().parseFromString(await response.text(), 'text/html')
      if (request.signal.aborted) return

      const nextResults = page.querySelector('[data-blog-results]')
      if (!nextResults) throw new Error('Missing blog results')

      if (!fromHistory && url.href !== window.location.href) {
        window.history.pushState(null, '', url)
      }
      const focusWasInResults = results.contains(document.activeElement)
      disposeResults()
      results.replaceChildren(...nextResults.childNodes)
      disposeResults = decorate(results)
      displayedUrl = url.href
      document.title = page.title
      onNavigate(url, page)
      if (focusWasInResults) results.querySelector('h1')?.focus({ preventScroll: true })
    } catch {
      if (request.signal.aborted) return
      if (fromHistory) window.location.replace(url)
      else window.location.assign(url)
    } finally {
      if (pending === request) {
        pending = undefined
        results.removeAttribute('aria-busy')
      }
    }
  }

  blog.addEventListener('click', (event) => {
    if (!(event instanceof MouseEvent) || event.defaultPrevented || event.button !== 0
        || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return

    const link = event.target instanceof Element
      ? event.target.closest<HTMLAnchorElement>('a[href][data-blog-link]')
      : null
    if (!link || (link.target && link.target !== '_self')) return
    const url = new URL(link.href)
    if (url.origin !== window.location.origin) return

    event.preventDefault()
    void navigate(url, false)
  })

  window.addEventListener('popstate', () => {
    void navigate(new URL(window.location.href), true)
  })
}
