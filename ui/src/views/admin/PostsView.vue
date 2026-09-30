<template>
  <div>
    <div class="d-flex justify-content-between align-items-center mb-3">
      <h2>Посты</h2>
      <button
        type="button"
        class="btn btn-primary"
        data-bs-toggle="modal"
        data-bs-target="#create-post"
      >
        Новый пост
      </button>
    </div>

    <AdminPagination :page="page" :pages="pages" base-path="/posts" />

    <PostForm modal-id="create-post" mode="create" @saved="refresh" />
    <PostForm modal-id="edit-post" mode="edit" :post="selectedPost" @saved="refresh" />
    <ConfirmDelete
      modal-id="delete-post"
      title="Удалить пост"
      message="Действительно удалить пост?"
      :item-id="selectedPost.id"
      :item-title="selectedPost.Title"
      :remove="(id) => apiService.deletePost(id)"
      success-text="Пост удалён"
      failure-text="Не удалось удалить пост"
      @deleted="refresh"
    />

    <div class="table-responsive" id="posts-table">
      <table class="table table-striped table-hover table-sm align-middle">
        <thead>
          <tr>
            <th scope="col">ID</th>
            <th scope="col">Создано</th>
            <th scope="col">Название</th>
            <th scope="col">Опубликовано</th>
            <th scope="col" class="text-end">Действия</th>
          </tr>
        </thead>
        <tbody>
          <TableStatus :loading="loading" :failed="failed" :rows="posts.length" :colspan="5" />
          <tr v-for="item in posts" :key="item.id">
            <td>{{ item.id }}</td>
            <td>
              <DateFormatter :date="item.Created" format-str="L"></DateFormatter>
            </td>
            <td>
              <a href="#" data-bs-toggle="modal" data-bs-target="#edit-post" @click="onSelect(item)">
                {{ item.Title }}
              </a>
            </td>
            <td>
              <span v-if="item.IsPublic" class="badge bg-success">Да</span>
              <span v-else class="badge bg-secondary">Нет</span>
            </td>
            <td class="text-end text-nowrap">
              <a
                :href="`/blog/${item.id}.html`"
                target="_blank"
                rel="noopener"
                class="btn btn-sm btn-outline-secondary me-1"
                title="Открыть на сайте"
              >
                <AppIcon icon="external-link-alt"></AppIcon>
              </a>
              <button
                type="button"
                class="btn btn-sm btn-outline-primary me-1"
                title="Редактировать"
                data-bs-toggle="modal"
                data-bs-target="#edit-post"
                @click="onSelect(item)"
              >
                <AppIcon icon="pen"></AppIcon>
              </button>
              <button
                type="button"
                class="btn btn-sm btn-outline-danger"
                title="Удалить"
                data-bs-toggle="modal"
                data-bs-target="#delete-post"
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
import DateFormatter from '@/components/DateFormatter.vue'
import PostForm from '@/components/admin/PostForm.vue'
import ConfirmDelete from '@/components/admin/ConfirmDelete.vue'
import AppIcon from '@/components/AppIcon.vue'
import AdminPagination from '@/components/admin/AdminPagination.vue'
import TableStatus from '@/components/admin/TableStatus.vue'
import { usePagedList } from '@/composables/usePagedList'
import { emptyPost, Query } from '@/models/blog'
import type { EditablePost } from '@/models/blog'

const route = useRoute()
const apiService = new ApiService()

const {
  items: posts,
  page,
  pages,
  loading,
  failed,
  refresh
} = usePagedList(
  (pageNum) => {
    const q = new Query()
    q.page = pageNum.toString()
    return apiService.getAdminPosts<EditablePost>(q)
  },
  () => parseInt(route.params.page as string) || 1,
  'Не удалось загрузить посты'
)

const selectedPost = ref<EditablePost>(emptyPost())

const onSelect = (p: EditablePost): void => {
  selectedPost.value = p
}
</script>

<style scoped lang="scss">
.shortDate {
  font-weight: normal;
  color: black;
}
</style>
