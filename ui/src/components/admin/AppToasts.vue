<template>
  <div class="toast-container position-fixed bottom-0 end-0 p-3">
    <div
      v-for="n in notifications"
      :key="n.id"
      class="toast show align-items-center border-0"
      :class="`text-bg-${n.kind}`"
      :role="n.kind === 'danger' ? 'alert' : 'status'"
      aria-atomic="true"
    >
      <div class="d-flex">
        <div class="toast-body">{{ n.text }}</div>
        <button
          type="button"
          class="btn-close btn-close-white me-2 m-auto"
          aria-label="Закрыть"
          @click="dismiss(n.id)"
        ></button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useNotify } from '@/composables/useNotify'

const { notifications, dismiss } = useNotify()
</script>
