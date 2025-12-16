<template>
  <dl>
    <div v-for="item in items" :key="item.cacheId">
      <dt>
        <a :href="item.link" v-html="item.htmlTitle"></a><br/>
        <span class="text-muted small" v-html="item.htmlFormattedUrl"></span>
      </dt>
      <dd class="small" v-html="item.htmlSnippet"></dd>
    </div>
  </dl>
</template>

<script setup lang="ts">
import { SearchResult } from '@/services/SearchService'

defineProps<{
  items: SearchResult[]
}>()
</script>