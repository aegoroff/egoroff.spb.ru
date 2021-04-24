<template>
  <div id="app">
    <vue-progress-bar></vue-progress-bar>
    <Navigation v-bind:navigation="navigation" v-bind:user="user"/>
    <Breadcrumbs v-bind:breadcrumbs="breadcrumbs" v-bind:title="title"/>
  </div>
</template>

<script lang="ts">
import 'reflect-metadata'
import { Component, Prop, Vue } from 'vue-property-decorator'
import Navigation, { Section } from '@/components/Navigation.vue'
import ApiService, { User } from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'
import Breadcrumbs from '@/components/Breadcrumbs.vue'

@Component({
  components: {
    Navigation,
    Breadcrumbs
  },
  providers: [ApiService]
})
export default class App extends Vue {
  private navigation!: Array<Section>
  private breadcrumbs!: Array<Section>
  @Prop() private user!: User
  @Prop() public title!: string
  @inject() private api!: ApiService

  constructor () {
    super()
    const nav = this.api.getNavigation()
    this.navigation = nav.sections
    this.breadcrumbs = nav.breadcrumbs
    this.api.getUser().then(r => {
      this.user = r
    })
  }
}
</script>

<style lang="scss">
#app {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
