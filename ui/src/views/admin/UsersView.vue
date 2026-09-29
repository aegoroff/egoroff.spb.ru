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
          <TableStatus :loading="loading" :failed="failed" :rows="users.length" :colspan="8" />
          <tr v-for="user in users" :key="`${user.provider}_${user.federatedId}`">
            <td>{{ user.federatedId }}</td>
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
import TableStatus from '@/components/admin/TableStatus.vue'
import { useNotify } from '@/composables/useNotify'

const users = ref<Array<FullUserInfo>>([])
const loading = ref(true)
const failed = ref(false)
const notify = useNotify()

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
    failed.value = true
    notify.error('Не удалось загрузить пользователей', error)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadUsers()
})
</script>

<style scoped lang="scss">
</style>