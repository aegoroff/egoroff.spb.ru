<template>
  <highlightjs :code="content" :language="lang" />
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
  },
  setup(props) {
    return {
      content: props.content,
      lang: props.lang
    }
  }
})
</script>