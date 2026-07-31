<template>
  <span itemprop="datePublished" :class="cssClass">{{ formatted }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { formatFromNow, formatLongDate } from '@/util'

const props = defineProps<{
  date: string
  formatStr: string
}>()

const fromNow = computed(() => props.formatStr === 'from-now')

const formatted = computed(() =>
  fromNow.value ? formatFromNow(props.date) : formatLongDate(props.date)
)

const cssClass = computed(() => (fromNow.value ? 'date-from-now' : 'shortDate'))
</script>
