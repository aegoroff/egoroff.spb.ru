<template>
  <dl>
    <div v-for="item in items" :key="item.cacheId">
      <dt>
        <a v-bind:href="item.link" v-html="item.htmlTitle">{{ item.htmlTitle }}</a><br/>
        <span class="text-muted small" v-html="item.htmlFormattedUrl">{{ item.htmlFormattedUrl }}</span>
      </dt>
      <dd class="small" v-html="item.htmlSnippet">{{ item.htmlSnippet }}</dd>
    </div>
  </dl>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import { SearchResult } from '@/services/SearchService.vue'

@Component
export default class SearchResulter extends Vue {
  @Prop() private items!: Array<SearchResult>
}
</script>
