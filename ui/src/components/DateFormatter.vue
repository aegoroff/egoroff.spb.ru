<template>
  <span itemprop="datePublished" :class="cssClass">{{ formatted }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import dayjs from 'dayjs'
import localizedFormat from 'dayjs/plugin/localizedFormat'
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/ru'

// dayjs config
dayjs.extend(localizedFormat)
dayjs.extend(relativeTime);
dayjs.locale('ru')

// props
const props = defineProps<{
  date: string
  formatStr: string
}>()

function formatnow(): boolean {
  return props.formatStr === "from-now";
}

const formatted = computed(() => {
  const d = dayjs(props.date);
  return formatnow() ? d.fromNow() : d.format(props.formatStr);
})

const cssClass = computed(() => formatnow() ? "date-from-now" : "shortDate")
</script>
