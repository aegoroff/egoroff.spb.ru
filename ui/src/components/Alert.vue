<template>
  <div>
    <div class="alert" :class="alertClass" role="alert">
      {{ content }}
      <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType, computed } from 'vue'

export default defineComponent({
  name: 'Alert',
  props: {
    type: {
      type: String as PropType<string>,
      default: 'success'
    },
    content: {
      type: String as PropType<string>,
      required: true
    }
  },
  setup(props) {
    const alertClass = computed(() => {
      const typeMap: Record<string, string> = {
        success: 'alert-success',
        danger: 'alert-danger',
        warning: 'alert-warning',
        info: 'alert-info'
      }
      return `alert ${typeMap[props.type] || 'alert-success'} alert-dismissible fade show`
    })

    return {
      alertClass
    }
  }
})
</script>