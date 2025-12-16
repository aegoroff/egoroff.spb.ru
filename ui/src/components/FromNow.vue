<template>
  <span class="date-from-now">{{ formatted }}</span>
</template>

<script lang="ts">
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/ru';
dayjs.extend(relativeTime);
dayjs.locale('ru');
import { computed, defineComponent } from 'vue'

export default defineComponent({
  name: 'FromNow',
  props: {
    date: {
      type: String,
      required: true
    }
  },
  setup(props) {
    const formatted = computed(() =>
      dayjs(props.date).fromNow()
    )
    return {
      formatted
    }
  }
})
</script>