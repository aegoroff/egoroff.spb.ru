<template>
  <div id="app">
    <vue3-progress-bar></vue3-progress-bar>
    <NavigationBar :navigation="navigation" :user="user"/>
    <BreadcrumbsBar :breadcrumbs="breadcrumbs" :title="title"/>
  </div>
</template>

<script setup lang="ts">
import {onMounted, ref} from 'vue'
import NavigationBar from '@/components/NavigationBar.vue'
import ApiService from '@/services/ApiService'
import {User as ApiUser} from '@/models/common'
import BreadcrumbsBar from '@/components/BreadcrumbsBar.vue'
import { Section } from './models/common';

defineProps<{
  title: string
}>()

const navigation = ref<Array<Section>>([])
const breadcrumbs = ref<Array<Section>>([])
const user = ref<ApiUser | null>(null)

onMounted(async () => {
  const apiService = new ApiService()
  const [nav, userResult] = await Promise.allSettled([
    apiService.getNavigation(),
    apiService.getUser()
  ])
  
  if (nav.status === 'fulfilled') {
    navigation.value = nav.value.sections
    breadcrumbs.value = nav.value.breadcrumbs
  }
  
  if (userResult.status === 'fulfilled') {
    user.value = userResult.value
  }
})
</script>

<style lang="scss">
#app {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
