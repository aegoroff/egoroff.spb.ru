<template>
  <div id="app">
    <Navigation v-bind:navigation="navigation"/>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import Navigation, { Section } from './components/Navigation.vue'
import axios from 'axios'

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

    axios.get<Array<Section>>('/api/v2/navigation.json').then(r => {
      this.navigation.push(...r.data)
    })
  }
}

</script>

<style lang="less">
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
