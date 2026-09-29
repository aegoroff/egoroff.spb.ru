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

    <PostForm modal-id="create-post" mode="create" />
    <PostForm modal-id="edit-post" mode="edit" :post="selectedPost" />
    <ConfirmDelete
      modal-id="delete-post"
      title="Удалить пост"
      message="Действительно удалить пост?"
      :item-id="selectedPostId"
      :item-title="selectedPost.Title"
      kind="post"
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
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import ApiService from '@/services/ApiService'
import DateFormatter from '@/components/DateFormatter.vue'
import PostForm from '@/components/admin/PostForm.vue'
import ConfirmDelete from '@/components/admin/ConfirmDelete.vue'
import AppIcon from '@/components/AppIcon.vue'
import { emitter } from '@/events'
import AdminPagination from '@/components/admin/AdminPagination.vue'
import TableStatus from '@/components/admin/TableStatus.vue'
import { useNotify } from '@/composables/useNotify'
import { EditablePost, Query } from '@/models/blog'

const route = useRoute()

const posts = ref<Array<EditablePost>>([])
const page = ref(1)
const pages = ref(1)
const selectedPost = ref<EditablePost>({
  Created: '',
  Modified: '',
  id: 0,
  Title: '',
  IsPublic: false,
  Markdown: false,
  Tags: [],
  Text: '',
  ShortText: ''
})
const selectedPostId = ref(0)
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
    const result = await apiService.getAdminPosts<EditablePost>(q)
    posts.value = result.result
    pages.value = result.pages
    page.value = result.page
  } catch (error) {
    failed.value = true
    notify.error('Не удалось загрузить посты', error)
  } finally {
    loading.value = false
  }
}

const onSelect = (p: EditablePost): void => {
  selectedPost.value = p
  selectedPostId.value = p.id
}

const refreshPosts = (): void => {
  update(page.value)
}

onMounted(() => {
  const routePage = parseInt(route.params.page as string) || 1
  update(routePage)

  emitter.on('postCreated', refreshPosts)
  emitter.on('postDeleted', refreshPosts)
  emitter.on('postUpdated', refreshPosts)
})

onUnmounted(() => {
  emitter.off('postCreated', refreshPosts)
  emitter.off('postDeleted', refreshPosts)
  emitter.off('postUpdated', refreshPosts)
})

// Watch route changes
watch(() => route.params.page, (newPage) => {
  const pageNum = parseInt(newPage as string) || 1
  update(pageNum)
})
</script>

<style scoped lang="scss">
.shortDate {
  font-weight: normal;
  color: black;
}
</style>
