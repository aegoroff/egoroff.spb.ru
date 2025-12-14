<template>
  <highlightjs :language="lang" :code="content" />
</template>
<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'Highlighter',
  props: {
    lang: {
      type: String,
      required: true
    },
    content: {
      type: String,
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