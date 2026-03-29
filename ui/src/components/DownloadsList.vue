<template>
  <h1>Загрузки</h1>
  <div v-for="folder in downloads" :key="folder.Title">
    <h3>{{ folder.Title }}</h3>
    <dl>
      <div v-for="f in folder.Files" :key="f.Blake3Hash">
        <dt itemscope itemtype="http://schema.org/SoftwareApplication">
          <a itemprop="downloadUrl" :href="f.Path">
            <font-awesome-icon icon="download"/>&nbsp;<span itemprop="name">{{ f.Title }}</span>
          </a>
        </dt>
        <dd>
          <small><span><strong>Платформа:</strong>&nbsp;Windows, x64</span></small><br/>
          <small><span><strong>Размер:</strong>&nbsp;{{ formatBytes(f.Size) }}</span></small><br/>
          <small><span><strong>Blake3:</strong>&nbsp;{{ f.Blake3Hash }}</span></small><br/>
        </dd>
      </div>
    </dl>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ApiService from '@/services/ApiService'
import { FilesContainer } from '@/models/portfolio'

const downloads = ref<Array<FilesContainer>>([])

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

onMounted(async () => {
  const apiService = new ApiService()
  try {
    const result = await apiService.getDownloadableFiles<FilesContainer>()
    downloads.value = result.result
  } catch (error) {
    console.error('Failed to fetch downloads:', error)
  }
})
</script>