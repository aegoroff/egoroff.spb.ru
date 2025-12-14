<template>
  <div id="blogPager" v-if="pages > 1">
    <nav aria-label="Page navigation">
      <ul class="pagination justify-content-center mb-0">
        <li :class="['page-item', { disabled: page === 1 }]">
          <a class="page-link" href="#" @click.prevent="goToPage(page - 1)">&larr; новее</a>
        </li>

        <li 
          v-for="n in pages" 
          :key="n" 
          :class="['page-item', { active: n === page }]"
        >
          <a class="page-link" :href="pageLinkGenerator(n)" @click.prevent="goToPage(n)">{{ n }}</a>
        </li>

        <li :class="['page-item', { disabled: page === pages }]">
          <a class="page-link" href="#" @click.prevent="goToPage(page + 1)">старее &rarr;</a>
        </li>
      </ul>
    </nav>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { emitter } from '@/main'

export default defineComponent({
  name: 'BlogPagination',
  props: {
    pages: {
      type: Number,
      required: true
    },
    page: {
      type: Number,
      required: true
    }
  },
  emits: ['update:page'],
  methods: {
    pageLinkGenerator(pageNum: number): string {
      const parts = window.location.hash.substring(1).split('&')
      let q = '#'
      for (const part of parts) {
        const elts = part.split('=')
        if (elts[0] !== 'page') {
          if (q !== '#') q += '&'
          q += elts[0] + '=' + elts[1]
        }
      }
      return `/blog/${q}&page=${pageNum}`
    },
    goToPage(pageNum: number) {
      if (pageNum < 1 || pageNum > this.pages) return
      emitter.emit('pageChanged', pageNum)
      this.$emit('update:page', pageNum)
    }
  }
})
</script>
