<template>
  <div id="app">
    <vue-progress-bar></vue-progress-bar>
    <Navigation v-bind:navigation="navigation"/>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import Navigation, { Section } from './components/Navigation.vue'
import axios from 'axios'

@Component({
  components: {
    Navigation
  }
})
export default class App extends Vue {
  private navigation: Array<Section>

  constructor () {
    super()

    this.navigation = new Array<Section>()

    this.$Progress.start()
    axios.get<Array<Section>>('/api/v2/navigation.json').then(r => {
      this.navigation.push(...r.data)
    }).finally(() => this.$Progress.finish())
  }
}
</script>

<style lang="scss">
#app {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
