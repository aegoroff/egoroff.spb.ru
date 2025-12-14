<template>
  <div>
    <dl itemscope itemtype="http://schema.org/BlogPosting" id="blogcontainer">
      <div v-for="post in posts" :key="post.id">
        <dt>
          <small>
            <DateFormatter :date="post.Created" format-str="LL"></DateFormatter>
          </small>&nbsp;
          <a itemprop="url" :href="'/blog/' + post.id + '.html'">
            <span itemprop="name">{{ post.id }}&nbsp;|&nbsp;{{ post.Title }}</span>
          </a>
        </dt>
        <dd itemprop="description" v-html="post.ShortText"></dd>
      </div>
    </dl>
    <BlogPagination v-if="pages > 1" :pages="pages" :page="currentPage" />
  </div>
</template>
<script lang="ts">
import { defineComponent, PropType, ref, onMounted } from 'vue'
import DateFormatter from '@/components/DateFomatter.vue'
import ApiService, { Query } from '@/services/ApiService'
import BlogPagination from '@/components/BlogPagination.vue'

export class Post {
  public Key!: string
  public Created!: string
  public id!: number
  public Title!: string
  public ShortText!: string
}

export default defineComponent({
  name: 'BlogAnnounces',
  components: {
    DateFormatter,
    BlogPagination
  },
  props: {
    q: {
      type: String as PropType<string>,
      default: ''
    }
  },
  setup(props) {
    const posts = ref<Array<Post>>([])
    const pages = ref(0)
    const currentPage = ref(1)
    
    const getQuery = (): Query => {
      const q = new Query()
      const parts = props.q.split('&')
      for (const part of parts) {
        const elts = part.split('=')
        if (elts[0] && elts[1]) {
          ;(q as any)[elts[0]] = decodeURIComponent(elts[1])
        }
      }
      return q
    }
    
    const updatePosts = async () => {
      const query = getQuery()
      const apiService = new ApiService()
      try {
        const result = await apiService.getPosts<Post>(query)
        posts.value = result.result
        pages.value = result.pages
        currentPage.value = result.page
      } catch (error) {
        console.error('Failed to fetch posts:', error)
      }
    }
    
    onMounted(() => {
      updatePosts()
    })
    
    return {
      posts,
      pages,
      currentPage,
      getQuery
    }
  }
})
</script>
<style scoped>
</style>