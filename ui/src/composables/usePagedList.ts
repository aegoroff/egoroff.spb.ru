import { ref, watch } from 'vue'
import type { Ref, WatchSource } from 'vue'
import type { ApiResult } from '@/services/ApiService'
import { useNotify } from '@/composables/useNotify'

export type PageFetcher<T> = (page: number) => Promise<ApiResult<T>>

export interface PagedList<T> {
  items: Ref<Array<T>>
  page: Ref<number>
  pages: Ref<number>
  loading: Ref<boolean>
  failed: Ref<boolean>
  /** Reloads the current page, e.g. after an item was saved or deleted. */
  refresh: () => Promise<void>
}

/**
 * Loads a server-paginated list and reloads it whenever `requestedPage` changes.
 * A failed load keeps the previous items, marks the list as failed and shows `errorText`.
 */
export function usePagedList<T>(
  fetchPage: PageFetcher<T>,
  requestedPage: WatchSource<number>,
  errorText: string
): PagedList<T> {
  const notify = useNotify()
  const items = ref([]) as Ref<Array<T>>
  const page = ref(1)
  const pages = ref(1)
  const loading = ref(true)
  const failed = ref(false)

  const load = async (pageNum: number): Promise<void> => {
    loading.value = true
    failed.value = false
    try {
      const result = await fetchPage(pageNum)
      items.value = result.result
      pages.value = result.pages
      page.value = result.page
    } catch (error) {
      failed.value = true
      notify.error(errorText, error)
    } finally {
      loading.value = false
    }
  }

  watch(requestedPage, load, { immediate: true })

  return { items, page, pages, loading, failed, refresh: () => load(page.value) }
}
