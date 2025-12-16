<template>
  <div id="app">
    <vue3-progress-bar></vue3-progress-bar>
    <Navigation :navigation="navigation" :user="user"/>
    <Breadcrumbs :breadcrumbs="breadcrumbs" :title="title"/>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import Navigation, { Section } from '@/components/Navigation.vue'
import ApiService, { User as ApiUser } from '@/services/ApiService'
import Breadcrumbs from '@/components/Breadcrumbs.vue'

const props = defineProps<{
  title: string
}>()

const navigation = ref<Array<Section>>([])
const breadcrumbs = ref<Array<Section>>([])
const user = ref<ApiUser | null>(null)

onMounted(async () => {
  const apiService = new ApiService()
  const nav = await apiService.getNavigation()
  navigation.value = nav.sections
  breadcrumbs.value = nav.breadcrumbs
  try {
    const userData = await apiService.getUser()
    user.value = userData
  } catch (error) {
    console.error('Failed to fetch user:', error)
  }
})
</script>

<style lang="scss">
#app {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>