<template>
  <div id="app">
    <vue-progress-bar></vue-progress-bar>
    <Navigation v-bind:navigation="navigation"/>
  </div>
</template>

<script lang="ts">
import 'reflect-metadata'
import { Component, Vue } from 'vue-property-decorator'
import Navigation, { Section } from './components/Navigation.vue'
import ApiService from './services/ApiService.vue'
import { inject } from 'vue-typescript-inject'

@Component({
  components: {
    Navigation
  },
  providers: [ApiService]
})
export default class App extends Vue {
  private navigation!: Array<Section>
  private breadcrumbs!: Array<Section>
  @inject() private api!: ApiService

  constructor () {
    super()
    this.navigation = this.api.getNavigation()
    this.breadcrumbs = this.api.getBreadcrumbs()
  }
}
</script>

<style lang="scss">
#app {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
