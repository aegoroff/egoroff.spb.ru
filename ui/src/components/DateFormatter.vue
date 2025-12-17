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


const formatted = computed(() => {
  if (props.formatStr === "from-now") {
    return dayjs(props.date).fromNow();
  } else {
    return dayjs(props.date).format(props.formatStr);
  }
})

const cssClass = computed(() => {
  if (props.formatStr === "from-now") {
    return "date-from-now";
  } else {
    return "shortDate";
  }
})
</script>
