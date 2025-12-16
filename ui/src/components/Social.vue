<template>
  <div>
    <a v-for="net in networks" :key="net" 
       :href="getShareUrl(net)" 
       class="btn btn-sm" 
       target="_blank">
      <font-awesome-icon :icon="['fab', net]"></font-awesome-icon>
    </a>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  url: string
  title: string
}>()

const networks = ['vk', 'telegram', 'whatsapp']
    
const getShareUrl = (network: string): string => {
  const encodedUrl = encodeURIComponent(props.url)
  const encodedTitle = encodeURIComponent(props.title)
  
  switch(network) {
    case 'vk':
      return `https://vk.com/share.php?url=${encodedUrl}&title=${encodedTitle}`
    case 'telegram':
      return `https://t.me/share/url?url=${encodedUrl}&text=${encodedTitle}`
    case 'whatsapp':
      return `https://api.whatsapp.com/send?text=${encodedTitle}%20${encodedUrl}`
    case 'facebook':
      return `https://www.facebook.com/sharer/sharer.php?u=${encodedUrl}`
    case 'twitter':
      return `https://twitter.com/intent/tweet?url=${encodedUrl}&text=${encodedTitle}`
    default:
      return '#'
  }
}
</script>