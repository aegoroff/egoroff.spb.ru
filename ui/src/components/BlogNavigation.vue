<template>
  <ul class="nav nav-tabs" role="tablist">
    <li class="nav-item" role="presentation">
      <button class="nav-link active" id="tags-tab" data-bs-toggle="tab"
              data-bs-target="#tags-content" type="button">Метки</button>
    </li>
    <li class="nav-item" role="presentation">
      <button class="nav-link" id="archive-tab" data-bs-toggle="tab"
              data-bs-target="#archive-content" type="button">Архив</button>
    </li>
  </ul>
  <div class="tab-content">
    <div class="tab-pane fade show active" id="tags-content">
      <Tags :tags="archive.tags" v-model:currentTag="currentTag" />
    </div>
    <div class="tab-pane fade" id="archive-content">
      <Chrono :years="archive.years"/>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ApiService from '@/services/ApiService'
import Tags from '@/components/Tags.vue'
import Chrono from '@/components/Chrono.vue'
import { Archive } from '@/models/blog'

const archive = ref<Archive>({
  tags: [],
  years: []
})
const currentTag = ref('')

onMounted(async () => {
  const apiService = new ApiService()
  try {
    archive.value = await apiService.getBlogArchive()
  } catch (error) {
    console.error('Failed to fetch blog archive:', error)
  }
})
</script>