<template>
  <b-container class="container" id="siteSearch" fluid="lg">

    <b-form inline @submit.prevent="search">
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
        <ul class="pagination" id="search-pager" data-bind="template: { name: 'pages', foreach: pages }">
        </ul>
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

  mounted (): void {
    if (this.query) {
      this.search()
    }
  }

  search (): void {
    const q = new SearchQuery()
    q.q = this.query
    q.key = this.key
    q.cx = this.cx
    this.service.search(q).then(success => {
      this.searchResult = success
    })
  }
}
</script>

<style scoped>

</style>
