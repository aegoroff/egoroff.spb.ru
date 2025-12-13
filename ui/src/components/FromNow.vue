<template>
  <span class="date-from-now">{{ format() }}</span>
</template>

<script lang="ts">
import moment from 'moment'
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  name: 'FromNow',
  props: {
    date: {
      type: String as PropType<string>,
      required: true
    }
  },
  setup(props) {
    const format = (): string => {
      return moment(props.date).locale('ru').fromNow()
    }
    
    return {
      format
    }
  }
})
</script>