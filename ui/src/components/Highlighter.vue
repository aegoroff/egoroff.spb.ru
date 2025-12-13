<template>
  <highlight-code :lang="lang">{{ content }}</highlight-code>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  name: 'Highlighter',
  props: {
    lang: {
      type: String as PropType<string>,
      required: true
    },
    content: {
      type: String as PropType<string>,
      required: true
    }
  }
})
</script>