<template>
    <code ref="codeEl" :class="`language-${lang}`">
      <slot>{{ content }}</slot>
    </code>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref, watch } from 'vue'
import hljs from 'highlight.js'

export default defineComponent({
  name: 'Highlighter',

  props: {
    lang: {
      type: String,
      required: true,
    },
    content: {
      type: String,
      required: true,
    },
  },

  setup(props) {
    const codeEl = ref<HTMLElement | null>(null)

    const highlight = () => {
      if (!codeEl.value) return
      codeEl.value.textContent = props.content
      hljs.highlightElement(codeEl.value)
    }

    onMounted(highlight)
    watch(() => props.content, highlight)

    return {
      codeEl,
    }
  },
})
</script>
