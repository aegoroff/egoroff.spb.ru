<template>
  <div>
    <h2 class="mb-4">Пользователи</h2>

    <div class="table-responsive">
      <table class="table table-striped table-hover table-sm">
        <thead>
          <tr>
            <th scope="col">ID</th>
            <th scope="col">Имя</th>
            <th scope="col">Логин</th>
            <th scope="col">Email</th>
            <th scope="col">Провайдер</th>
            <th scope="col">Админ</th>
            <th scope="col">Проверен</th>
            <th scope="col">Дата регистрации</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.id">
            <td>{{ user.id }}</td>
            <td>{{ user.name }}</td>
            <td>{{ user.username }}</td>
            <td>{{ user.email }}</td>
            <td>{{ user.provider }}</td>
            <td>
              <span v-if="user.admin" class="badge bg-success">Да</span>
              <span v-else class="badge bg-secondary">Нет</span>
            </td>
            <td>
              <span v-if="user.verified" class="badge bg-success">Да</span>
              <span v-else class="badge bg-secondary">Нет</span>
            </td>
            <td>{{ formatDate(user.created) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ApiService from '@/services/ApiService'
import { FullUserInfo } from '@/models/common'

const users = ref<Array<FullUserInfo>>([])

const formatDate = (dateString: string): string => {
  const date = new Date(dateString)
  return date.toLocaleDateString('ru-RU', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

const loadUsers = async () => {
  const apiService = new ApiService()
  try {
    const result = await apiService.getUsers<FullUserInfo>()
    users.value = result.result
  } catch (error) {
    console.error('Failed to fetch users:', error)
  }
}

onMounted(() => {
  loadUsers()
})
</script>

<style scoped lang="scss">
</style>