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
      const date = moment(props.date, moment.ISO_8601)
      if (!date.isValid()) {
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