<template>
  <div>
    <ShareNetwork class="btn btn-sm" v-for="net in networks" :key="net" :network="net" :url="url"
                  :title="title">
      <font-awesome-icon :icon="['fab', net]"></font-awesome-icon>
    </ShareNetwork>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType, ref, onMounted } from 'vue'

export default defineComponent({
  name: 'Social',
  props: {
    url: {
      type: String as PropType<string>,
      required: true
    },
    title: {
      type: String as PropType<string>,
      required: true
    }
  },
  setup() {
    const networks = ref<string[]>([])
    
    onMounted(() => {
      networks.value = ['vk']
    })
    
    return {
      networks
    }
  }
})
</script>