<template>
  <div class="container">
    <vue3-progress-bar></vue3-progress-bar>
    <div class="row">
      <div class="col-12 col-md-3 col-lg-2 mb-3">
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
    <app-toasts/>
  </div>
</template>

<script setup lang="ts">
import MainMenu from '@/components/admin/MainMenu.vue'
import AppToasts from '@/components/admin/AppToasts.vue'
</script>

<style scoped>

</style>
