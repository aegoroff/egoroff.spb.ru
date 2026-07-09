<template>
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
  <Teleport to="#blogPager">
    <BlogPagination v-if="pages > 1" :pages="pages" :page="page" />
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import DateFormatter from '@/components/DateFormatter.vue'
import ApiService from '@/services/ApiService'
import BlogPagination from '@/components/BlogPagination.vue'
import { Post, Query } from '@/models/blog'

const props = withDefaults(defineProps<{
  q?: string
}>(), {
  q: ''
})

const posts = ref<Array<Post>>([])
const pages = ref(0)
const page = ref(1)

const getQuery = (): Query => {
  const q = new Query()
  const parts = props.q.split('&')

  for (const part of parts) {
    if (!part) {
      continue
    }
    const elts = part.split('=')
    Reflect.set(q, elts[0], elts[1])
  }
  return q
}

const loadPosts = async (): Promise<void> => {
  const apiService = new ApiService()
  const result = await apiService.getPosts<Post>(getQuery())
  posts.value = result.result
  pages.value = result.pages
  page.value = result.page
}

onMounted(() => {
  loadPosts().catch((error: unknown) => {
    console.error('Failed to fetch blog posts:', error)
  })
})
</script>

<style scoped>
</style>
