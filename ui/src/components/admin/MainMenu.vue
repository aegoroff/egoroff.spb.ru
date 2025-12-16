<template>
  <nav>
    <ul class="nav nav-pills flex-column">
      <li class="nav-item">
        <router-link class="nav-link" :class="{ active: $route.path.startsWith('/posts') }" to="/posts">Блог</router-link>
      </li>
      <li class="nav-item">
        <router-link class="nav-link" :class="{ active: $route.path.startsWith('/downloads') }" to="/downloads">Загрузки</router-link>
      </li>
      <li class="nav-item">
        <router-link class="nav-link" :class="{ active: $route.path.startsWith('/users') }" to="/users">Пользователи</router-link>
      </li>
    </ul>
  </nav>
</template>

<script setup lang="ts">
</script>