<template>
  <nav>
    <ul class="nav nav-pills flex-row flex-md-column">
      <li class="nav-item">
        <router-link class="nav-link" :class="{ active: $route.path === '/' }" to="/">Дашборд</router-link>
      </li>
      <li class="nav-item">
        <router-link class="nav-link" :class="{ active: $route.path.startsWith('/posts') }" to="/posts">Блог</router-link>
      </li>
      <li class="nav-item">
        <router-link class="nav-link" :class="{ active: $route.path.startsWith('/downloads') }" to="/downloads">Загрузки</router-link>
      </li>
      <li class="nav-item">
        <router-link class="nav-link" :class="{ active: $route.path.startsWith('/users') }" to="/users">Пользователи</router-link>
      </li>
      <li class="nav-item mt-md-3">
        <a class="nav-link" href="/" target="_blank" rel="noopener">
          <AppIcon icon="external-link-alt"></AppIcon> На сайт
        </a>
      </li>
    </ul>
  </nav>
</template>

<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
</script>