<template>
  <nav class="navbar navbar-expand-lg navbar-dark navbar-custom navbar-static-top">
    <div class="container">
      <a class="navbar-brand" href="/">egoroff.spb.ru</a>

      <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#nav-collapse">
        <span class="navbar-toggler-icon"></span>
      </button>

      <div class="collapse navbar-collapse" id="nav-collapse">
        <ul class="navbar-nav me-auto">
          <li class="nav-item" v-for="section in navigation" :key="section.id">
            <a class="nav-link" :class="{ active: section.active }" :href="'/' + section.id + '/'"
               :title="section.descr">
              <font-awesome-icon :icon="section.icon"/>
              {{ section.title }}
            </a>
          </li>
          <li class="nav-item">
            <form class="d-flex" action="/search/" method="GET">
              <input class="form-control form-control-sm me-2" name="q" type="search" 
                     placeholder="Введите текст для поиска" required>
              <button class="btn btn-sm btn-primary" type="submit">Поиск</button>
            </form>
          </li>
        </ul>

        <ul class="navbar-nav">
          <li class="nav-item" v-if="!user || !user.authenticated">
            <a class="nav-link" href="/login">
              <font-awesome-icon icon="sign-in-alt"/>
              Войти
            </a>
          </li>
          <li class="nav-item dropdown" v-if="user && user.authenticated">
            <a class="nav-link dropdown-toggle" href="#" role="button" data-bs-toggle="dropdown">
              <font-awesome-icon :icon="['fab', user.provider]"></font-awesome-icon>&nbsp;
              <em>{{ user.loginOrName }}</em>
            </a>
            <ul class="dropdown-menu dropdown-menu-end">
              <li><a class="dropdown-item" href="/profile">
                <font-awesome-icon icon="user"/>
                Профиль
              </a></li>
              <li v-if="user && user.admin"><a class="dropdown-item" href="/admin">
                <font-awesome-icon icon="tools"/>
                Админка
              </a></li>
              <li><hr class="dropdown-divider"></li>
              <li><a class="dropdown-item" href="/logout">
                <font-awesome-icon icon="sign-out-alt"/>
                Выйти
              </a></li>
            </ul>
          </li>
        </ul>
      </div>
    </div>
  </nav>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'
import { User } from '@/services/ApiService.vue'

export class Section {
  public id!: string
  public title!: string
  public class!: string
  public icon!: string
  public active!: boolean
  public descr!: string
}

export default defineComponent({
  name: 'Navigation',
  props: {
    navigation: {
      type: Array as PropType<Section[]>,
      required: true
    },
    user: {
      type: Object as PropType<User | null>,
      default: null
    }
  }
})
</script>

<style scoped lang="scss">
.navbar-custom {
  background-color: rgb(104, 148, 216);
}
</style>