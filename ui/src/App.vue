<template>
  <div id="app">
    <Navigation/>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import Navigation from './components/Navigation.vue'

@Component({
  components: {
    Navigation: Navigation
  }
})
export default class App extends Vue {}
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
