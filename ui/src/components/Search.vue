<template>
  <div class="container" id="siteSearch">
    <div class="row">
      <div class="col-lg-8">
        <label class="sr-only" for="q">форма поиска</label>
        <input id="q" name="q" class="form-control" type="text" v-model="query"/>
      </div>
      <div class="col-lg-4">
        <a id="start_search" class="btn btn-primary" v-on:click="search"><i class="icon" data-label="search"></i> Искать</a>
      </div>
    </div>
    <div class="row" v-if="searchResult">
      <div class="col-lg-12">
        <br/>
        <div class="text-muted">Результатов: примерно
          <span>{{ searchResult.searchInformation.formattedTotalResults }}</span>
          <nobr> (<span>{{ searchResult.searchInformation.formattedSearchTime }}</span> cек.)</nobr>
          <p/></div>
        <SearchResulter v-bind:items="searchResult.items"/>
        <ul class="pagination" id="search-pager" data-bind="template: { name: 'pages', foreach: pages }">
        </ul>
      </div>
    </div>
  </div>
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
