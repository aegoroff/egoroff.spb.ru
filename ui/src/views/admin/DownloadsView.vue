<template>
  <div>
    <div class="d-flex justify-content-between align-items-center mb-3">
      <h2>Загрузки</h2>
      <button class="btn btn-primary" data-bs-toggle="modal" data-bs-target="#create-download">
        Новая загрузка
      </button>
    </div>

    <AdminPagination :page="page" :pages="pages" base-path="/downloads" />

    <DownloadForm modal-id="edit-download" mode="edit" :download="selectedDownload" />
    <DownloadForm modal-id="create-download" mode="create" />
    <ConfirmDelete
      modal-id="delete-download"
      title="Удалить загрузку"
      message="Действительно удалить загрузку?"
      :item-id="selectedDownloadId"
      :item-title="selectedDownload.title"
      kind="download"
    />

    <div class="table-responsive" id="downloads-table">
      <table class="table table-striped table-hover table-sm align-middle">
        <thead>
          <tr>
            <th scope="col">ID</th>
            <th scope="col">Название</th>
            <th scope="col" class="text-end">Действия</th>
          </tr>
        </thead>
        <tbody>
          <TableStatus :loading="loading" :failed="failed" :rows="downloads.length" :colspan="3" />
          <tr v-for="item in downloads" :key="item.id">
            <td>{{ item.id }}</td>
            <td>
              <a href="#" data-bs-toggle="modal" data-bs-target="#edit-download" @click="onSelect(item)">
                {{ item.title }}
              </a>
            </td>
            <td class="text-end text-nowrap">
              <button
                type="button"
                class="btn btn-sm btn-outline-primary me-1"
                title="Редактировать"
                data-bs-toggle="modal"
                data-bs-target="#edit-download"
                @click="onSelect(item)"
              >
                <AppIcon icon="pen"></AppIcon>
              </button>
              <button
                type="button"
                class="btn btn-sm btn-outline-danger"
                title="Удалить"
                data-bs-toggle="modal"
                data-bs-target="#delete-download"
                @click="onSelect(item)"
              >
                <AppIcon icon="trash-alt"></AppIcon>
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import ApiService from '@/services/ApiService'
import AppIcon from '@/components/AppIcon.vue'
import { emitter } from '@/events'
import AdminPagination from '@/components/admin/AdminPagination.vue'
import TableStatus from '@/components/admin/TableStatus.vue'
import { useNotify } from '@/composables/useNotify'
import DownloadForm from '@/components/admin/DownloadForm.vue'
import ConfirmDelete from '@/components/admin/ConfirmDelete.vue'
import { Download } from '@/models/portfolio'
import { Query } from '@/models/blog'

const route = useRoute()

const downloads = ref<Array<Download>>([])
const page = ref(1)
const pages = ref(1)
const selectedDownload = ref<Download>({
  id: 0,
  title: ''
})
const selectedDownloadId = ref(0)
const loading = ref(true)
const failed = ref(false)
const notify = useNotify()

const update = async (pageNum: number): Promise<void> => {
  const q = new Query()
  q.page = pageNum.toString()
  q.limit = '10'

  const apiService = new ApiService()
  loading.value = true
  failed.value = false
  try {
    const result = await apiService.getDownloads<Download>(q)
    downloads.value = result.result
    pages.value = result.pages
    page.value = result.page
  } catch (error) {
    failed.value = true
    notify.error('Не удалось загрузить список загрузок', error)
  } finally {
    loading.value = false
  }
}

const onSelect = (d: Download): void => {
  selectedDownload.value = d
  selectedDownloadId.value = d.id
}

const refreshDownloads = (): void => {
  update(page.value)
}

onMounted(() => {
  const routePage = parseInt(route.params.page as string) || 1
  update(routePage)

  emitter.on('downloadDeleted', refreshDownloads)
  emitter.on('downloadCreated', refreshDownloads)
  emitter.on('downloadUpdated', refreshDownloads)
})

onUnmounted(() => {
  emitter.off('downloadDeleted', refreshDownloads)
  emitter.off('downloadCreated', refreshDownloads)
  emitter.off('downloadUpdated', refreshDownloads)
})

// Watch route changes
watch(() => route.params.page, (newPage) => {
  const pageNum = parseInt(newPage as string) || 1
  update(pageNum)
})
</script>

<style scoped lang="scss">
</style>
