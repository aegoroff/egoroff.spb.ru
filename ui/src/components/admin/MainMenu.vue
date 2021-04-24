<template>
  <nav>
    <b-nav vertical pills>
      <b-nav-item to="/" exact exact-active-class="active">Dashboard</b-nav-item>
      <b-nav-item to="/users" exact exact-active-class="active">Пользователи</b-nav-item>
      <b-nav-item to="/posts" exact exact-active-class="active">Блог</b-nav-item>
    </b-nav>
  </nav>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'

@Component
export default class MainMenu extends Vue {
}
</script>

<style scoped lang="scss">

</style>
