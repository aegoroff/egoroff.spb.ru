<template>
  <highlightjs :language="lang" :code="content" />
</template>
<script lang="ts">
import { defineComponent, PropType } from 'vue'
import { Vue3Highlightjs } from 'vue3-highlightjs'

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