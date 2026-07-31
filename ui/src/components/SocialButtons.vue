<template>
  <div>
    <ShareNetwork
      v-for="net in networks"
      :key="net"
      :network="net"
      :url="url"
      :title="title"
      v-slot="{ share }"
    >
      <a href="#" class="btn btn-sm" @click.prevent="share">
        <font-awesome-icon :icon="['fab', net]"></font-awesome-icon>
      </a>
    </ShareNetwork>
  </div>
</template>

<script setup lang="ts">
import { ShareNetwork } from "vue3-social-sharing";

defineProps<{
  url: string;
  title: string;
}>();

const networks = ["vk", "telegram"] as const;
</script>
