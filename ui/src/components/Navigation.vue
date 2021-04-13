<template>
  <b-navbar toggleable="lg" type="dark" class="navbar-custom navbar-static-top">
    <div class="container">
      <b-navbar-brand href="/">egoroff.spb.ru</b-navbar-brand>

      <b-navbar-toggle target="nav-collapse"></b-navbar-toggle>

      <b-collapse id="nav-collapse" is-nav>
        <b-navbar-nav fill>
          <b-nav-item
            v-for="section in navigation"
            :key="section.id"
            :active="section.active"
            v-bind:href="'/'+section.id+'/'">
            <font-awesome-icon v-bind:icon="section.icon"/>
            {{ section.title }}
          </b-nav-item>
          <b-nav-form action="/search/" method="GET">
            <b-form-input name="q" size="sm" class="mr-md-2" placeholder="Введите текст для поиска" required></b-form-input>
            <b-button size="sm" class="my-2 my-sm-0" type="submit" variant="primary">Поиск</b-button>
          </b-nav-form>
        </b-navbar-nav>

        <!-- Right aligned nav items -->
        <b-navbar-nav class="ml-auto">
          <b-nav-item href="/login" active right v-if="!user || !user.authenticated">
            <font-awesome-icon icon="sign-in-alt"/>
            Войти
          </b-nav-item>
          <b-nav-item-dropdown right v-if="user && user.authenticated">
            <!-- Using 'button-content' slot -->
            <template #button-content>
              <em>{{user.loginOrName}}</em>
            </template>
            <b-dropdown-item href="/profile">Профиль</b-dropdown-item>
            <b-dropdown-item href="/logout">
              <font-awesome-icon icon="sign-out-alt"/>
              Выйти
            </b-dropdown-item>
          </b-nav-item-dropdown>
        </b-navbar-nav>
      </b-collapse>
    </div>
  </b-navbar>

</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import { User } from '@/services/ApiService.vue'

export class Section {
  public id!: string
  public title!: string
  public class!: string
  public icon!: string
  public active!: boolean
}

@Component
export default class Navigation extends Vue {
  @Prop() private navigation!: Array<Section>
  @Prop() private user!: User
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
.navbar-custom {
  background-color: rgb(104, 148, 216);
}
</style>
