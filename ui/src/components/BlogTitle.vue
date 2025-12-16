<template>
  <small id="blogSmallTitle">{{ text }}</small>
</template>

<script setup lang="ts">
defineProps<{
  text: string
}>()
</script>