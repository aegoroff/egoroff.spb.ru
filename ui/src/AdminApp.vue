<template>
  <div class="container">
    <div class="row">
      <div class="col-2">
        <main-menu/>
      </div>
      <div class="col">
        <div class="main-panel">
          <div class="content-wrapper">
            <router-view></router-view>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import MainMenu from '@/components/admin/MainMenu.vue'

export default defineComponent({
  name: 'AdminApp',
  components: {
    MainMenu
  }
})
</script>

<style scoped>

</style>