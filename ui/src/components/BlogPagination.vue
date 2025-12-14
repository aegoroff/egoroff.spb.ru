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
import { defineComponent, computed } from 'vue'
import { emitter } from '@/main'
import { removeHash } from '@/util'

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
        // Получаем текущий хэш
        const currentHash = window.location.hash.substring(1)
        const params = new URLSearchParams(currentHash)
        
        // Удаляем параметр page из параметров
        params.delete('page')
        
        // Если это не первая страница, добавляем параметр page
        if (newPage > 1) {
          params.set('page', newPage.toString())
        }
        
        // Формируем новый хэш
        const newParams = params.toString()
        
        if (newParams) {
          window.location.hash = '#' + newParams
        } else {
          // Если нет параметров, полностью убираем хэш
          removeHash()
        }
        
        // Эмитируем событие для обновления постов
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