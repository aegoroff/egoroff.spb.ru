<template>
  <span class="date-from-now">{{ formatted }}</span>
</template>

<script lang="ts">
import moment from 'moment'
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
      moment(props.date).locale('ru').fromNow()
    )
    return {
      formatted
    }
  }
})
</script>