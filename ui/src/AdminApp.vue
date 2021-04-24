<template>
  <div class="container">
    <b-row>
      <b-col cols="2">
        <main-menu/>
      </b-col>
      <b-col>
        <div class="main-panel">
          <div class="content-wrapper">
            <router-view></router-view>
          </div>
        </div>
      </b-col>
    </b-row>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import MainMenu from '@/components/admin/MainMenu.vue'

@Component({
  components: {
    MainMenu
  }
})
export default class AdminApp extends Vue {
}
</script>

<style scoped>

</style>
