<template>
  <small id="blogSmallTitle">{{ text }}</small>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  name: 'BlogTitle',
  props: {
    text: {
      type: String as PropType<string>,
      required: true
    }
  }
})
</script>