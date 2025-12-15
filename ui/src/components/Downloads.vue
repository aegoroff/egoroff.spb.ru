<template>
  <div class="col-lg-6">
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
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue'
import ApiService from '@/services/ApiService'

export class Downloadable {
  public Title!: string
  public Path!: string
  public FileName!: string
  public Blake3Hash!: string
  public Size!: number
}

export class FilesContainer {
  public Title!: string
  public Files!: Array<Downloadable>
}

export default defineComponent({
  name: 'Downloads',
  setup() {
    const downloads = ref<Array<FilesContainer>>([])
    
    onMounted(async () => {
      const apiService = new ApiService()
      try {
        const result = await apiService.getDownloadableFiles<FilesContainer>()
        downloads.value = result.result
      } catch (error) {
        console.error('Failed to fetch downloads:', error)
      }
    })
    
    const formatBytes = (bytes: number): string => {
      if (bytes === 0) return '0 Bytes'
      const k = 1024
      const sizes = ['Bytes', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }
    
    return {
      downloads,
      formatBytes
    }
  }
})
</script>