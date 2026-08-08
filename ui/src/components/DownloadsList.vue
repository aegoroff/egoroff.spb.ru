<template>
  <h1>Загрузки</h1>
  <div v-for="folder in downloads" :key="folder.Title">
    <h3>{{ folder.Title }}</h3>
    <dl>
      <div v-for="f in visibleFiles(folder)" :key="f.Blake3Hash">
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
    <p v-if="hiddenCount(folder) > 0">
      <a href="#" @click.prevent="toggleExpanded(folder.Title)">
        {{ isExpanded(folder.Title)
          ? 'Свернуть'
          : `Показать ещё ${hiddenCount(folder)}` }}
      </a>
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ApiService from '@/services/ApiService'
import { Downloadable, FilesContainer } from '@/models/portfolio'

const PREVIEW_COUNT = 3

const downloads = ref<Array<FilesContainer>>([])
const expandedFolders = ref<Record<string, boolean>>({})

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const isExpanded = (folderTitle: string): boolean =>
  !!expandedFolders.value[folderTitle]

const toggleExpanded = (folderTitle: string): void => {
  expandedFolders.value[folderTitle] = !expandedFolders.value[folderTitle]
}

const hiddenCount = (folder: FilesContainer): number =>
  Math.max(0, folder.Files.length - PREVIEW_COUNT)

const visibleFiles = (folder: FilesContainer): Array<Downloadable> => {
  if (isExpanded(folder.Title) || folder.Files.length <= PREVIEW_COUNT) {
    return folder.Files
  }
  return folder.Files.slice(0, PREVIEW_COUNT)
}

onMounted(async () => {
  const apiService = new ApiService()
  try {
    const result = await apiService.getDownloadableFiles<FilesContainer>()
    downloads.value = result.result.map((folder) => ({
      ...folder,
      Files: [...folder.Files].reverse(),
    }))
  } catch (error) {
    console.error('Failed to fetch downloads:', error)
  }
})
</script>
