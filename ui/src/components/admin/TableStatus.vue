<template>
  <tr v-if="rows === 0">
    <td v-if="loading" :colspan="colspan" class="text-center py-4">
      <div class="spinner-border spinner-border-sm text-secondary me-2" role="status"></div>
      Загрузка…
    </td>
    <td v-else-if="failed" :colspan="colspan" class="text-center text-danger py-4">
      Не удалось загрузить данные
    </td>
    <td v-else :colspan="colspan" class="text-center text-body-secondary py-4">Нет записей</td>
  </tr>
</template>

<script setup lang="ts">
// Existing rows stay visible during a reload; the global progress bar signals it.
defineProps<{
  loading: boolean
  failed: boolean
  rows: number
  colspan: number
}>()
</script>
