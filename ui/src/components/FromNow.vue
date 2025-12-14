<template>
  <span class="date-from-now">{{ formattedDate }}</span>
</template>

<script lang="ts">
import moment from 'moment'
import 'moment/locale/ru'
import { defineComponent, PropType, computed } from 'vue'

export default defineComponent({
  name: 'FromNow',
  props: {
    date: {
      type: String as PropType<string>,
      required: true
    }
  },
  setup(props) {
    const formattedDate = computed(() => {
      const date = moment(props.date)
      if (!date.isValid()) {
        // Попробуем разные форматы, если стандартный не работает
        const alternativeFormats = [
          'YYYY-MM-DDTHH:mm:ss.SSSZ',
          'YYYY-MM-DD HH:mm:ss',
          'YYYY-MM-DD',
          'DD.MM.YYYY',
          'DD/MM/YYYY'
        ]
        
      for (const format of alternativeFormats) {
        const parsedDate = moment(props.date, format)
        if (parsedDate.isValid()) {
          return parsedDate.locale('ru').fromNow()
        }
      }
        return props.date
      }
      return date.locale('ru').fromNow()
    })
    
    return {
      formattedDate
    }
  }
})
</script>