<template>
  <ul class="nav nav-tabs" role="tablist">
    <li class="nav-item" role="presentation">
      <button class="nav-link" :class="{ active: !currentYear }" id="tags-tab" data-bs-toggle="tab"
              data-bs-target="#tags-content" type="button">Метки</button>
    </li>
    <li class="nav-item" role="presentation">
      <button class="nav-link" :class="{ active: currentYear }" id="archive-tab" data-bs-toggle="tab"
              data-bs-target="#archive-content" type="button">Архив</button>
    </li>
  </ul>
  <div class="tab-content">
    <div class="tab-pane fade" :class="{ show: !currentYear, active: !currentYear }" id="tags-content">
      <TagsControl :tags="archive.tags" :current-tag="currentTag" />
    </div>
    <div class="tab-pane fade" :class="{ show: currentYear, active: currentYear }" id="archive-content">
      <ChronoControl :years="archive.years" :current-year="currentYear" :current-month="currentMonth" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import ApiService from '@/services/ApiService'
import TagsControl from '@/components/TagsControl.vue'
import ChronoControl from '@/components/ChronoControl.vue'
import type { Archive } from '@/models/blog'

const props = defineProps<{ search: string }>()

const archive = ref<Archive>({
  tags: [],
  years: []
})
const filter = computed(() => new URLSearchParams(props.search))
const currentTag = computed(() => filter.value.get('tag') ?? '')
const currentYear = computed(() => currentTag.value ? 0 : Number(filter.value.get('year')))
const currentMonth = computed(() => currentYear.value ? Number(filter.value.get('month')) : 0)

onMounted(async () => {
  const apiService = new ApiService()
  try {
    archive.value = await apiService.getBlogArchive()
  } catch (error) {
    console.error('Failed to fetch blog archive:', error)
  }
})
</script>
