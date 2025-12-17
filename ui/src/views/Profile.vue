<template>
  <div class="container" id="userProfile">
    <div class="pb-2 mt-4 mb-2 border-bottom">
      <h1>
        <small class="float-end" title="Connected Accounts">
          <AppIcon lib="fab" :icon="user.provider"></AppIcon>
        </small>
        {{ user.name }}
      </h1>
    </div>

    <form @submit.prevent="update">
      <div class="row">
        <div class="col">
          <div class="mb-3">
            <label for="login" class="form-label">Логин:</label>
            <input id="login" class="form-control" readonly v-model="user.username">
          </div>
          <div class="mb-3">
            <label for="userName" class="form-label">Имя:</label>
            <input id="userName" class="form-control" required v-model="user.name">
          </div>

          <div class="mb-3">
            <label for="userEmail" class="form-label">Электропочта:</label>
            <input
              id="userEmail"
              class="form-control"
              v-model="user.email"
              type="email"
              placeholder="Введите адрес электропочты"
              required
            >
            <div class="form-text">С вашей электропочтой я никогда и ни с кем не поделюсь</div>
          </div>
        </div>

        <div class="col">
          <div class="mb-3" v-if="user.avatarUrl">
            <label for="avatar" class="form-label d-block">Аватар:</label>

            <img class="img-thumbnail mb-2" id="avatar" :src="user.avatarUrl" width="180"/>

            <p class="text-muted mb-0">
              Изменить на
              <a href="//gravatar.com" target="_blank">Gravatar</a>
            </p>
          </div>
          <div class="mb-3" v-if="!user.avatarUrl">
            <label for="newAvatarUrl" class="form-label">Аватар:</label>
            <input id="newAvatarUrl" class="form-control" v-model="newAvatarUrl">
          </div>
        </div>
      </div>
      <div class="row">
        <div class="col">
          <button class="btn btn-primary btn-lg" type="submit">Обновить профиль</button>
        </div>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ApiService from '@/services/ApiService'
import AppIcon from '@/components/AppIcon.vue'
import { FullUserInfo } from '@/models/access'

const user = ref<FullUserInfo>({
  id: 0,
  admin: false,
  created: '',
  avatarUrl: '',
  email: '',
  name: '',
  username: '',
  verified: false,
  provider: ''
})
const newAvatarUrl = ref('')

const readProfile = async (): Promise<void> => {
  const apiService = new ApiService()
  try {
    user.value = await apiService.getFullUserInfo()
  } catch (error) {
    console.error('Failed to fetch user info:', error)
  }
}

onMounted(() => {
  readProfile()
})

const update = (): void => {
  if (newAvatarUrl.value) {
    user.value.avatarUrl = newAvatarUrl.value
  }

  const apiService = new ApiService()
  apiService.updateFullUserInfo(user.value)

  // Сброс поля для нового аватара
  newAvatarUrl.value = ''
}
</script>

<style scoped>

</style>