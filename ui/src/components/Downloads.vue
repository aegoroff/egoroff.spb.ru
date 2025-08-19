<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'

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

@Component({
  providers: [ApiService]
})
export default class Downloads extends Vue {
  @Prop() private downloads!: Array<FilesContainer>
  @inject() private api!: ApiService

  mounted (): void {
    this.api.getDownloadableFiles<FilesContainer>().then(x => {
      this.downloads = x.result
    })
  }
}
</script>

<template>
  <div class="col-lg-6">
    <h1>Загрузки</h1>
    <div v-for="folder in downloads" :key="folder.Title">
      <h3> {{ folder.Title }}</h3>
      <dl>
        <div v-for="f in folder.Files" :key="f.Blake3Hash">
          <dt itemscope itemtype="http://schema.org/SoftwareApplication">
            <a itemprop="downloadUrl" v-bind:href="f.Path">
              <i class="icon" data-label="download"></i>&nbsp;<span itemprop="name">{{ f.Title }}</span>
            </a>
          </dt>
          <dd>
            <small><span><strong>Платформа:</strong>&nbsp;Windows, x64</span></small><br/>
            <small><span><strong>Размер:</strong>&nbsp;{{ f.Size | bytes }}</span></small><br/>
            <small><span><strong>Blake3:</strong>&nbsp;{{ f.Blake3Hash }}</span></small><br/>
          </dd>
        </div>
      </dl>
    </div>
  </div>
</template>

<style scoped>

</style>
