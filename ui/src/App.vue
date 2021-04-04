<template>
  <div id="app">
    <Navigation v-bind:navigation="navigation"/>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import Navigation, { Section } from './components/Navigation.vue'
import axios from 'axios'
import NProgress from 'nprogress'

@Component({
  components: {
    Navigation: Navigation
  }
})
export default class App extends Vue {
  private navigation: Array<Section>

  constructor () {
    super()

    this.navigation = new Array<Section>()

    NProgress.start()
    axios.get<Array<Section>>('/api/v2/navigation.json').then(r => {
      this.navigation.push(...r.data)
    }).finally(() => NProgress.done())
  }
}
</script>

<style lang="scss">
@import '~nprogress/nprogress.css';

#app {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
