<template>
  <div id="app">
    <vue-progress-bar></vue-progress-bar>
    <Navigation :navigation="navigation" :user="user"/>
    <Breadcrumbs :breadcrumbs="breadcrumbs" :title="title"/>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue'
import Navigation, { Section } from '@/components/Navigation.vue'
import ApiService, { User as ApiUser, Nav } from '@/services/ApiService.vue'
import Breadcrumbs from '@/components/Breadcrumbs.vue'

export default defineComponent({
  name: 'App',
  components: {
    Navigation,
    Breadcrumbs
  },
  props: {
    title: {
      type: String,
      required: true
    }
  },
  setup(props) {
    const navigation = ref<Array<Section>>([])
    const breadcrumbs = ref<Array<Section>>([])
    const user = ref<ApiUser | null>(null)
    
    onMounted(async () => {
      // Инициализация навигации
      const apiService = new ApiService()
      const nav = apiService.getNavigation()
      navigation.value = nav.sections
      breadcrumbs.value = nav.breadcrumbs
      
      // Получение информации о пользователе
      try {
        const userData = await apiService.getUser()
        user.value = userData
      } catch (error) {
        console.error('Failed to fetch user:', error)
      }
    })
    
    return {
      navigation,
      breadcrumbs,
      user
    }
  }
})
</script>

<style lang="scss">
#app {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>