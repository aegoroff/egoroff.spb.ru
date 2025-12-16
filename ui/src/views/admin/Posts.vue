<template>
  <div>
    <nav v-if="pages > 1">
      <ul class="pagination justify-content-center" id="posts-pager">
        <li class="page-item" :class="{ disabled: page === 1 }">
          <router-link :to="`/posts/${page - 1}`" class="page-link">Назад</router-link>
        </li>
        <li class="page-item" v-for="p in pageNumbers" :key="p" :class="{ active: p === page }">
          <router-link :to="`/posts/${p}`" class="page-link">{{ p }}</router-link>
        </li>
        <li class="page-item" :class="{ disabled: page === pages }">
          <router-link :to="`/posts/${page + 1}`" class="page-link">Вперед</router-link>
        </li>
      </ul>
    </nav>
    
    <EditPost id="edit-post" :post="selectedPost"></EditPost>
    <DeletePost id="delete-post" :postId="selectedPostId"></DeletePost>
    
    <div class="table-responsive" id="posts-table">
      <table class="table table-striped table-hover table-sm">
        <thead>
          <tr>
            <th scope="col">-</th>
            <th scope="col">ID</th>
            <th scope="col">Создано</th>
            <th scope="col">Название</th>
            <th scope="col">Опубликовано</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in posts" :key="item.id">
            <td>
              <a href="#" data-bs-toggle="modal" data-bs-target="#delete-post" @click="onSelect(item)">
                <AppIcon icon="trash-alt"></AppIcon>
              </a>
            </td>
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
              <span v-if="item.IsPublic">Да</span>
              <span v-else>Нет</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ApiService, { Query } from '@/services/ApiService'
import DateFormatter from '@/components/DateFormatter.vue'
import EditPost, { Post } from '@/components/admin/EditPost.vue'
import DeletePost from '@/components/admin/DeletePost.vue'
import AppIcon from '@/components/AppIcon.vue'
import { emitter } from '@/main'

export default defineComponent({
  name: 'Posts',
  components: {
    DateFormatter,
    EditPost,
    DeletePost,
    AppIcon
  },
  setup() {
    const route = useRoute()
    const router = useRouter()
    
    const posts = ref<Array<Post>>([])
    const page = ref(1)
    const pages = ref(1)
    const selectedPost = ref<Post>({
      Created: '',
      id: 0,
      Title: '',
      IsPublic: false,
      Markdown: false,
      Tags: [],
      Text: '',
      ShortText: ''
    })
    const selectedPostId = ref(0)
    
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
        const result = await apiService.getAdminPosts<Post>(q)
        posts.value = result.result
        pages.value = result.pages
        page.value = result.page
      } catch (error) {
        console.error('Failed to fetch posts:', error)
      }
    }
    
    const onSelect = (p: Post): void => {
      selectedPost.value = p
      selectedPostId.value = p.id
    }
    
    onMounted(() => {
      // Инициализация из роута
      const routePage = parseInt(route.params.page as string) || 1
      update(routePage)
      
      // Подписываемся на события
      emitter.on('postDeleted', () => {
        update(page.value)
      })
    })
    
    // Отслеживаем изменения роута
    watch(() => route.params.page, (newPage) => {
      const pageNum = parseInt(newPage as string) || 1
      update(pageNum)
    })
    
    return {
      posts,
      page,
      pages,
      selectedPost,
      selectedPostId,
      pageNumbers,
      update,
      onSelect
    }
  }
})
</script>

<style scoped lang="scss">
.shortDate {
  font-weight: normal;
  color: black;
}
</style>