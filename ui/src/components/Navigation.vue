<template>
  <b-navbar toggleable="lg" type="dark" class="navbar-custom">
    <b-navbar-brand href="#">Админка</b-navbar-brand>

    <b-navbar-toggle target="nav-collapse"></b-navbar-toggle>

    <b-collapse id="nav-collapse" is-nav>
      <b-navbar-nav>
        <b-nav-item
          v-for="section in navigation"
          :key="section.id"
          v-bind:href="section.id">
          <font-awesome-icon v-bind:icon="section.icon" />
          {{ section.title }}
        </b-nav-item>
      </b-navbar-nav>

      <!-- Right aligned nav items -->
      <b-navbar-nav class="ml-auto">
        <b-nav-form>
          <b-form-input size="sm" class="mr-sm-2" placeholder="Search"></b-form-input>
          <b-button size="sm" class="my-2 my-sm-0" type="submit">Search</b-button>
        </b-nav-form>

        <b-nav-item-dropdown right>
          <!-- Using 'button-content' slot -->
          <template #button-content>
            <em>User</em>
          </template>
          <b-dropdown-item href="#">Profile</b-dropdown-item>
          <b-dropdown-item href="#">Sign Out</b-dropdown-item>
        </b-nav-item-dropdown>
      </b-navbar-nav>
    </b-collapse>
  </b-navbar>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'

export class Section {
  public id: string
  public title: string
  public class: string
  public icon: string

  constructor () {
    this.id = ''
    this.title = ''
    this.class = ''
    this.icon = ''
  }
}

@Component
export default class Navigation extends Vue {
  @Prop() private navigation!: Array<Section>
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
.navbar-custom {
  background-color: rgb(104, 148, 216);
  .navbar-brand, .navbar-text, .navbar-nav .nav-link {
    color: white;
  }
}
</style>
