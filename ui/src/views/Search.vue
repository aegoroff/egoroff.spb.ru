<template>
  <div class="container" id="siteSearch">
    <form class="d-flex mb-3" @submit.prevent="newSearch">
      <div class="row w-100">
        <div class="col-8">
          <input
            id="search-control"
            class="form-control w-100"
            v-model="localQuery"
            placeholder="Введите текст для поиска"
            required
          >
        </div>
        <div class="col">
          <button class="btn btn-primary" type="submit">
            <i class="fas fa-search"></i> Искать
          </button>
        </div>
      </div>
    </form>

    <div class="row" v-if="searchResult">
      <div class="col">
        <div class="text-muted mb-3">Результатов: примерно
          <span>{{ searchResult.searchInformation.formattedTotalResults }}</span>
          <span> (<span>{{ searchResult.searchInformation.formattedSearchTime }}</span> сек.)</span>
        </div>
        <SearchResulter :items="searchResult.items"/>
        <nav v-if="pages > 1 || page > 1">
          <ul class="pagination">
            <li class="page-item" :class="{ disabled: page === 1 }">
              <a class="page-link" href="#" @click.prevent="changePage(page - 1)">Назад</a>
            </li>
            <li class="page-item" v-for="p in pageNumbers" :key="p" :class="{ active: p === page }">
              <a class="page-link" href="#" @click.prevent="changePage(p)">{{ p }}</a>
            </li>
            <li class="page-item" :class="{ disabled: page === pages }">
              <a class="page-link" href="#" @click.prevent="changePage(page + 1)">Вперед</a>
            </li>
          </ul>
        </nav>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted } from 'vue'
import SearchService, { GoogleSearch, SearchQuery } from '@/services/SearchService'
import SearchResulter from '@/components/SearchResulter.vue'
import { removeHash } from '@/util'

const ItemsPerPage = 10

export default defineComponent({
  name: 'Search',
  components: { SearchResulter },
  props: {
    query: {
      type: String,
      default: ''
    },
    apiKey: {
      type: String,
      default: ''
    },
    cx: {
      type: String,
      default: ''
    }
  },
  setup(props) {
    const searchResult = ref<GoogleSearch | null>(null)
    const localQuery = ref(props.query)
    const pages = ref(0)
    const page = ref(1)
    
    const pageNumbers = computed(() => {
      const numbers = []
      const start = Math.max(1, page.value - 2)
      const end = Math.min(pages.value, page.value + 2)
      
      for (let i = start; i <= end; i++) {
        numbers.push(i)
      }
      return numbers
    })
    
    onMounted(() => {
      if (localQuery.value) {
        search()
      }
    })
    
    const newSearch = (): void => {
      removeHash()
      page.value = 1
      search()
    }
    
    const search = async (): Promise<void> => {
      if (document.location.hash) {
        const hashPage = parseInt(document.location.hash.substring(1), 10)
        if (!isNaN(hashPage) && hashPage > 0) {
          page.value = hashPage
        }
      }
      
      const q = new SearchQuery()
      q.q = localQuery.value
      q.key = props.apiKey
      q.cx = props.cx
      q.start = (page.value - 1) * ItemsPerPage + 1
      
      const service = new SearchService()
      try {
        const result = await service.search(q)
        searchResult.value = result
        
        const totalResults = parseInt(result.searchInformation.totalResults, 10)
        pages.value = Math.ceil(totalResults / ItemsPerPage)
      } catch (error) {
        console.error('Search failed:', error)
      }
    }
    
    const changePage = (newPage: number): void => {
      if (newPage >= 1 && newPage <= pages.value && newPage !== page.value) {
        page.value = newPage
        document.location.hash = `#${newPage}`
        search()
      }
    }
    
    return {
      searchResult,
      localQuery,
      pages,
      page,
      pageNumbers,
      newSearch,
      search,
      changePage
    }
  }
})
</script>