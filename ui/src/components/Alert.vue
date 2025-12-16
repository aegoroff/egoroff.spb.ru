<template>
  <div>
    <div class="alert" :class="alertClass" role="alert">
      {{ content }}
      <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  type?: string
  content: string
}>()

const alertClass = computed(() => {
  const typeMap: Record<string, string> = {
    success: 'alert-success',
    danger: 'alert-danger',
    warning: 'alert-warning',
    info: 'alert-info'
  }
  return `alert ${typeMap[props.type || 'success']} alert-dismissible fade show`
})
</script>