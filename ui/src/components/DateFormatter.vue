<template>
  <span itemprop="datePublished" class="shortDate">{{ formatted }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import dayjs from 'dayjs'
import localizedFormat from 'dayjs/plugin/localizedFormat'
import 'dayjs/locale/ru'

// dayjs config
dayjs.extend(localizedFormat)
dayjs.locale('ru')

// props
const props = defineProps<{
  date: string
  formatStr: string
}>()

// computed
const formatted = computed(() =>
  dayjs(props.date).format(props.formatStr)
)
</script>
