<template>
  <span itemprop="datePublished" class="shortDate">{{ formatted }}</span>
</template>
<script lang="ts">
import moment from 'moment'
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
      moment(props.date).locale('ru').format(props.formatStr)
    )
    return {
      formatted
    }
  }
})
</script>