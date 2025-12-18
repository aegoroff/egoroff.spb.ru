<template>
    <code ref="codeEl" :class="`language-${language}`">
      <slot>{{ content }}</slot>
    </code>
</template>

<script setup lang="ts">
import { onMounted, ref, watch, computed } from 'vue'
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

const langMap = new Map<string, string>([
  ["asm", "x86asm"],
  ["hq", "cs"],
  ["parser", "parser3"],
  ["php", "parser3"],
]);

const language = computed(() => langMap.get(props.lang) ?? props.lang)

onMounted(highlight)
watch(() => props.content, highlight)
</script>
