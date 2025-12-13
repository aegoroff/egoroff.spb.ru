<template>
  <div id="blogPager">
    <nav v-if="pages > 1">
      <ul class="pagination justify-content-center">
        <li class="page-item" :class="{ disabled: page === 1 }">
          <a class="page-link" href="#" @click.prevent="changePage(page - 1)">&larr; новее</a>
        </li>
        <li class="page-item" v-for="p in pageNumbers" :key="p" :class="{ active: p === page }">
          <a class="page-link" href="#" @click.prevent="changePage(p)">{{ p }}</a>
        </li>
        <li class="page-item" :class="{ disabled: page === pages }">
          <a class="page-link" href="#" @click.prevent="changePage(page + 1)">старее &rarr;</a>
        </li>
      </ul>
    </nav>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType, computed } from 'vue'
import { emitter } from '@/main'

export default defineComponent({
  name: 'BlogPagination',
  props: {
    pages: {
      type: Number as PropType<number>,
      required: true
    },
    page: {
      type: Number as PropType<number>,
      required: true
    }
  },
  setup(props) {
    const pageNumbers = computed(() => {
      const numbers = []
      const start = Math.max(1, props.page - 2)
      const end = Math.min(props.pages, props.page + 2)
      
      for (let i = start; i <= end; i++) {
        numbers.push(i)
      }
      return numbers
    })

    const changePage = (newPage: number): void => {
      if (newPage >= 1 && newPage <= props.pages && newPage !== props.page) {
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
        
        if (q === '#') {
          q += 'page=' + newPage
        } else {
          q += '&page=' + newPage
        }
        
        window.location.hash = q
        emitter.emit('pageChanged', newPage)
      }
    }

    return {
      pageNumbers,
      changePage
    }
  }
})
</script>