<template>
  <div>
    <button class="btn btn-primary w-100 mb-3" data-bs-toggle="modal" data-bs-target="#create-download">
      Создать загрузку
    </button>

    <nav v-if="pages > 1">
      <ul class="pagination justify-content-center" id="downloads-pager">
        <li class="page-item" :class="{ disabled: page === 1 }">
          <router-link :to="`/downloads/${page - 1}`" class="page-link">Назад</router-link>
        </li>
        <li class="page-item" v-for="p in pageNumbers" :key="p" :class="{ active: p === page }">
          <router-link :to="`/downloads/${p}`" class="page-link">{{ p }}</router-link>
        </li>
        <li class="page-item" :class="{ disabled: page === pages }">
          <router-link :to="`/downloads/${page + 1}`" class="page-link">Вперед</router-link>
        </li>
      </ul>
    </nav>

    <EditDownload id="edit-download" :download="selectedDownload"></EditDownload>
    <CreateDownload id="create-download"></CreateDownload>
    <DeleteDownload id="delete-download" :downloadId="selectedDownloadId"></DeleteDownload>

    <div class="table-responsive" id="downloads-table">
      <table class="table table-striped table-hover table-sm">
        <thead>
          <tr>
            <th scope="col">-</th>
            <th scope="col">ID</th>
            <th scope="col">Название</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in downloads" :key="item.id">
            <td>
              <a href="#" data-bs-toggle="modal" data-bs-target="#delete-download" @click="onSelect(item)">
                <AppIcon icon="trash-alt"></AppIcon>
              </a>
            </td>
            <td>{{ item.id }}</td>
            <td>
              <a href="#" data-bs-toggle="modal" data-bs-target="#edit-download" @click="onSelect(item)">
                {{ item.title }}
              </a>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import ApiService, { Query } from '@/services/ApiService'
import AppIcon from '@/components/AppIcon.vue'
import { emitter } from '@/main'
import EditDownload from '@/components/admin/EditDownload.vue'
import { Download } from '@/models/portfolio'
import DeleteDownload from '@/components/admin/DeleteDownload.vue'
import CreateDownload from '@/components/admin/CreateDownload.vue'

const route = useRoute()

const downloads = ref<Array<Download>>([])
const page = ref(1)
const pages = ref(1)
const selectedDownload = ref<Download>({
  id: 0,
  title: ''
})
const selectedDownloadId = ref(0)

const pageNumbers = computed(() => {
  const numbers = []
  const start = Math.max(1, page.value - 2)
  const end = Math.min(pages.value, page.value + 2)

  for (let i = start; i <= end; i++) {
    numbers.push(i)
  }
  return numbers
})

const update = async (pageNum: number): Promise<void> => {
  const q = new Query()
  q.page = pageNum.toString()
  q.limit = '10'

  const apiService = new ApiService()
  try {
    const result = await apiService.getDownloads<Download>(q)
    downloads.value = result.result
    pages.value = result.pages
    page.value = result.page
  } catch (error) {
    console.error('Failed to fetch downloads:', error)
  }
}

const onSelect = (d: Download): void => {
  selectedDownload.value = d
  selectedDownloadId.value = d.id
}

onMounted(() => {
  const routePage = parseInt(route.params.page as string) || 1
  update(routePage)

  emitter.on('downloadDeleted', () => {
    update(page.value)
  })

  emitter.on('downloadCreated', () => {
    update(page.value)
  })
})

// Отслеживаем изменения роута
watch(() => route.params.page, (newPage) => {
  const pageNum = parseInt(newPage as string) || 1
  update(pageNum)
})
</script>

<style scoped lang="scss">
</style>
