<template>
  <span class="date-from-now">{{ formatted }}</span>
</template>

<script setup lang="ts">
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/ru';
dayjs.extend(relativeTime);
dayjs.locale('ru');
import { computed } from 'vue'

const props = defineProps<{
  date: string
}>()

const formatted = computed(() =>
  dayjs(props.date).fromNow()
)
</script>