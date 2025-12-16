<template>
  <i>
    <font-awesome-icon v-if="!lib" :icon="icon"/>
    <font-awesome-icon v-if="lib" :icon="[lib, icon]"/>
  </i>
</template>

<script setup lang="ts">
defineProps<{
  icon: string
  lib?: string
}>()
</script>