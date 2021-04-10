<template>
  <dl>
    <div v-for="item in items" :key="item.cacheId">
      <dt>
        <a v-bind:href="item.link">{{ item.htmlTitle }}</a><br/>
        <span class="text-muted small">{{ item.htmlFormattedUrl }}</span>
      </dt>
      <dd class="small">{{ item.htmlSnippet }}</dd>
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
