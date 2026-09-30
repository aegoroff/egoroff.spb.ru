<template>
  <div>
    <div class="d-flex justify-content-between align-items-center mb-3">
      <h2>Загрузки</h2>
      <button class="btn btn-primary" data-bs-toggle="modal" data-bs-target="#create-download">
        Новая загрузка
      </button>
    </div>

    <AdminPagination :page="page" :pages="pages" base-path="/downloads" />

    <DownloadForm
      modal-id="edit-download"
      mode="edit"
      :download="selectedDownload"
      @saved="refresh"
    />
    <DownloadForm modal-id="create-download" mode="create" @saved="refresh" />
    <ConfirmDelete
      modal-id="delete-download"
      title="Удалить загрузку"
      message="Действительно удалить загрузку?"
      :item-id="selectedDownload.id"
      :item-title="selectedDownload.title"
      :remove="(id) => apiService.deleteDownload(id)"
      success-text="Загрузка удалена"
      failure-text="Не удалось удалить загрузку"
      @deleted="refresh"
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
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import ApiService from '@/services/ApiService'
import AppIcon from '@/components/AppIcon.vue'
import AdminPagination from '@/components/admin/AdminPagination.vue'
import TableStatus from '@/components/admin/TableStatus.vue'
import DownloadForm from '@/components/admin/DownloadForm.vue'
import ConfirmDelete from '@/components/admin/ConfirmDelete.vue'
import { usePagedList } from '@/composables/usePagedList'
import { emptyDownload } from '@/models/portfolio'
import type { Download } from '@/models/portfolio'
import type { Query } from '@/models/blog'

const route = useRoute()
const apiService = new ApiService()

const {
  items: downloads,
  page,
  pages,
  loading,
  failed,
  refresh
} = usePagedList(
  (pageNum) => {
    const q: Query = {}
    q.page = pageNum.toString()
    return apiService.getDownloads(q)
  },
  () => parseInt(route.params.page as string) || 1,
  'Не удалось загрузить список загрузок'
)

const selectedDownload = ref<Download>(emptyDownload())

const onSelect = (d: Download): void => {
  selectedDownload.value = d
}
</script>

<style scoped lang="scss">
</style>
