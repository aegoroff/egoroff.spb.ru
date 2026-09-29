<template>
  <nav v-if="pages > 1">
    <ul class="pagination justify-content-center">
      <li class="page-item" :class="{ disabled: page === 1 }">
        <span v-if="page === 1" class="page-link">Назад</span>
        <router-link v-else :to="`${basePath}/${page - 1}`" class="page-link">Назад</router-link>
      </li>
      <li class="page-item" v-for="p in pageNumbers" :key="p" :class="{ active: p === page }">
        <router-link :to="`${basePath}/${p}`" class="page-link">{{ p }}</router-link>
      </li>
      <li class="page-item" :class="{ disabled: page === pages }">
        <span v-if="page === pages" class="page-link">Вперед</span>
        <router-link v-else :to="`${basePath}/${page + 1}`" class="page-link">Вперед</router-link>
      </li>
    </ul>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  page: number
  pages: number
  basePath: string
}>()

const pageNumbers = computed(() => {
  const numbers = []
  const start = Math.max(1, props.page - 2)
  const end = Math.min(props.pages, props.page + 2)
  for (let i = start; i <= end; i++) {
    numbers.push(i)
  }
  return numbers
})
</script>
