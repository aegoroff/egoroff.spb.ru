<template>
  <span itemprop="datePublished" class="shortDate">{{ format() }}</span>
</template>
<script lang="ts">
import moment from 'moment'
import { defineComponent, PropType } from 'vue'
export default defineComponent({
  name: 'DateFormatter',
  props: {
    date: {
      type: String as PropType<string>,
      required: true
    },
    formatStr: {
      type: String as PropType<string>,
      required: true
    }
  },
  setup(props) {
    const format = (): string => {
      return moment(props.date).locale('ru').format(props.formatStr)
    }
    return {
      format
    }
  }
})
</script>