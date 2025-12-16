<template>
  <span itemprop="datePublished" class="shortDate">{{ formatted }}</span>
</template>
<script lang="ts">
import dayjs from 'dayjs';
import localizedFormat from 'dayjs/plugin/localizedFormat' 
import 'dayjs/locale/ru';
dayjs.extend(localizedFormat);
dayjs.locale('ru');

import { computed, defineComponent } from 'vue'
export default defineComponent({
  name: 'DateFormatter',
  props: {
    date: {
      type: String,
      required: true
    },
    formatStr: {
      type: String,
      required: true
    }
  },
  setup(props) {
    const formatted = computed(() =>
      dayjs(props.date).format(props.formatStr)
    )
    return {
      formatted
    }
  }
})
</script>