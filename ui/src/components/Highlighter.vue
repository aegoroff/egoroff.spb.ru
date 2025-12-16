<template>
    <code ref="codeEl" :class="`language-${lang}`">
      <slot>{{ content }}</slot>
    </code>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import hljs from 'highlight.js'

const props = defineProps<{
  lang: string
  content: string
}>()

const codeEl = ref<HTMLElement | null>(null)

const highlight = () => {
  if (!codeEl.value) return
  codeEl.value.textContent = props.content
  hljs.highlightElement(codeEl.value)
}

onMounted(highlight)
watch(() => props.content, highlight)
</script>