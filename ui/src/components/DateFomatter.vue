<template>
  <span itemprop="datePublished" class="shortDate">{{ format() }}</span>
</template>
<script lang="ts">
import moment from 'moment'
import 'moment/locale/ru'
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
      const date = moment(props.date, moment.ISO_8601)
      if (!date.isValid()) {
        return props.date
      }
      return date.locale('ru').format(props.formatStr)
    }
    return {
      format
    }
  }
})
</script>