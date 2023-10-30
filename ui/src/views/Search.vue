<template>
  <b-container class="container" id="siteSearch" fluid="lg">
    <b-form inline @submit.prevent="newSearch">
      <b-form-row class="w-100">
        <b-col cols="8">
          <b-form-input
            id="search-control"
            class="mb-2 mr-md-2 mb-md-0 w-100"
            v-model="query"
            placeholder="Введите текст для поиска"
            required
          ></b-form-input>
        </b-col>
        <b-col>
          <b-button variant="primary" type="submit"><i class="icon" data-label="search"></i> Искать</b-button>
        </b-col>
      </b-form-row>
    </b-form>

    <b-row v-if="searchResult">
      <b-col>
        <br/>
        <div class="text-muted">Результатов: примерно
          <span>{{ searchResult.searchInformation.formattedTotalResults }}</span>
          <nobr> (<span>{{ searchResult.searchInformation.formattedSearchTime }}</span> cек.)</nobr>
          <p/></div>
        <SearchResulter v-bind:items="searchResult.items"/>
        <b-pagination-nav
          v-if="pages > 1 || page > 1"
          base-url="#"
          v-model="page"
          :number-of-pages="pages"
          @change="search"
          :per-page="10"></b-pagination-nav>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import 'reflect-metadata'
import { Component, Prop, Vue } from 'vue-property-decorator'
import SearchService, { GoogleSearch, SearchQuery } from '@/services/SearchService.vue'
import { inject } from 'vue-typescript-inject'
import SearchResulter from '@/components/SearchResulter.vue'
import { removeHash } from '@/util'

const ItemsPerPage = 10

@Component({
  components: { SearchResulter },
  providers: [SearchService]
})
export default class Search extends Vue {
  @inject() private service!: SearchService
  @Prop() private searchResult!: GoogleSearch
  @Prop() private query!: string
  @Prop() private key!: string
  @Prop() private cx!: string
  @Prop() private pages!: number
  @Prop() private page!: number

  mounted (): void {
    if (this.query) {
      this.search()
    }
  }

  newSearch (): void {
    removeHash()
    this.search()
  }

  search (): void {
    if (document.location.hash) {
      this.page = parseInt(document.location.hash.substring(0, 1), 10)
      if (isNaN(this.page) || this.page === 0) {
        this.page = 1
      }
    } else {
      this.page = 1
    }
    const q = new SearchQuery()
    q.q = this.query
    q.key = this.key
    q.cx = this.cx
    q.start = (this.page - 1) * ItemsPerPage + 1
    this.service.search(q).then(success => {
      this.searchResult = success
      this.pages = parseInt(this.searchResult.searchInformation.totalResults, 10)
      this.pages = Math.ceil(this.pages / ItemsPerPage)
    })
  }
}
</script>
