<template>
  <i>
    <font-awesome-icon v-if="!lib" :icon="icon"/>
    <font-awesome-icon v-if="lib" :icon="[lib, icon]"/>
  </i>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  name: 'AppIcon',
  props: {
    icon: {
      type: String as PropType<string>,
      required: true
    },
    lib: {
      type: String as PropType<string>,
      required: false,
      default: ''
    }
  }
})
</script>