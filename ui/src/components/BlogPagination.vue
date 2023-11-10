<template>
  <div id="blogPager">
    <b-pagination-nav
      v-if="pages > 1"
      v-bind:number-of-pages="pages"
      v-model="page"
      hide-goto-end-buttons
      :link-gen="pageLinkGenerator"
      @change="update"
      align="center"
    >
      <template #prev-text>&larr; новее</template>
      <template #next-text>старее &rarr;</template>
    </b-pagination-nav>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import { bus } from '@/main'

@Component
export default class BlogPagination extends Vue {
  @Prop() private pages!: number
  @Prop() private page!: number

  pageLinkGenerator (pageNum: number): string {
    const parts = window.location.hash.substring(1).split('&')

    let q = '#'
    for (const part of parts) {
      const elts = part.split('=')
      if (elts[0] !== 'page') {
        if (q !== '#') {
          q += '&'
        }
        q += elts[0] + '=' + elts[1]
      }
    }

    return `/blog/${q}&page=${pageNum}`
  }

  update (page: number): void {
    bus.$emit('pageChanged', page)
  }
}
</script>
