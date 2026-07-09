<template>
  <div>
    <h2 class="mb-4">Панель управления</h2>

    <div class="row mb-4">
      <div class="col-md-4 mb-3">
        <div class="card bg-primary text-white h-100">
          <div class="card-body">
            <div class="d-flex justify-content-between align-items-center">
              <div>
                <h6 class="card-title mb-2">Посты в блоге</h6>
                <h2 class="mb-0">{{ stats.posts_count }}</h2>
              </div>
              <div class="display-4">
                <font-awesome-icon icon="book" />
              </div>
            </div>
            <router-link to="/posts" class="text-white text-decoration-none mt-3 d-inline-block">
              <font-awesome-icon icon="arrow-right" /> Перейти к постам
            </router-link>
          </div>
        </div>
      </div>

      <div class="col-md-4 mb-3">
        <div class="card bg-success text-white h-100">
          <div class="card-body">
            <div class="d-flex justify-content-between align-items-center">
              <div>
                <h6 class="card-title mb-2">Загрузки</h6>
                <h2 class="mb-0">{{ stats.downloads_count }}</h2>
              </div>
              <div class="display-4">
                <font-awesome-icon icon="download" />
              </div>
            </div>
            <router-link to="/downloads" class="text-white text-decoration-none mt-3 d-inline-block">
              <font-awesome-icon icon="arrow-right" /> Перейти к загрузкам
            </router-link>
          </div>
        </div>
      </div>

      <div class="col-md-4 mb-3">
        <div class="card bg-info text-white h-100">
          <div class="card-body">
            <div class="d-flex justify-content-between align-items-center">
              <div>
                <h6 class="card-title mb-2">Пользователи</h6>
                <h2 class="mb-0">{{ stats.users_count }}</h2>
              </div>
              <div class="display-4">
                <font-awesome-icon icon="users" />
              </div>
            </div>
            <router-link to="/users" class="text-white text-decoration-none mt-3 d-inline-block">
              <font-awesome-icon icon="arrow-right" /> Перейти к пользователям
            </router-link>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ApiService from '@/services/ApiService'
import { DashboardStats } from '@/models/dashboard'

const stats = ref<DashboardStats>(new DashboardStats())

const loadStats = async () => {
  const apiService = new ApiService()
  try {
    stats.value = await apiService.getDashboardStats()
  } catch (error) {
    console.error('Failed to fetch dashboard stats:', error)
  }
}

onMounted(() => {
  loadStats()
})
</script>

<style scoped lang="scss">
.card {
  transition: transform 0.2s;
  
  &:hover {
    transform: translateY(-2px);
  }
}
</style>