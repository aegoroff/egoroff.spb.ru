<template>
  <div>
    <b-pagination-nav
      id="downloads-pager"
      aria-controls="downloads-table"
      v-if="pages > 1"
      v-bind:number-of-pages="pages"
      v-model="page"
      @page-click="onChange"
      align="center"
      :link-gen="pageLinkGenerator"
      use-router
    ></b-pagination-nav>
    <EditDownload id="edit-download" :download="selectedDownload"></EditDownload>
    <DeleteDownload id="delete-download" :downloadId="selectedDownloadId"></DeleteDownload>
    <b-table-lite responsive small striped hover :items="downloads" :fields="fields" id="downloads-table">
      <template #cell(title)="data">
        <a v-b-modal.edit-download href="#" @click="onSelect(data.item)">{{ data.value }}</a>
      </template>
      <template #cell(-)="data">
        <a v-b-modal.delete-download href="#" @click="onSelect(data.item)">
          <AppIcon icon="trash-alt"></AppIcon>
        </a>
      </template>
    </b-table-lite>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService, { Query } from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'
import { BvEvent } from 'bootstrap-vue'
import AppIcon from '@/components/AppIcon.vue'
import { bus } from '@/main'
import EditDownload, { Download } from '@/components/admin/EditDownload.vue'
import DeleteDownload from '@/components/admin/DeleteDownload.vue'

@Component({
  components: {
    EditDownload,
    DeleteDownload,
    AppIcon
  },
  providers: [ApiService]
})
export default class Downloads extends Vue {
  @inject() private api!: ApiService
  @Prop() private downloads!: Array<Download>
  @Prop() private page!: number
  @Prop() private pages!: number
  @Prop() private selectedDownload!: Download
  @Prop() private selectedDownloadId!: number
  private fields = ['-', 'id', 'title']

  constructor () {
    super()
    this.update(1)
    bus.$on('downloadDeleted', () => {
      this.update(this.page)
    })
  }

  update (page: number): void {
    const q = new Query()
    q.page = page.toString()
    q.limit = '10'
    this.api.getDownloads<Download>(q).then(x => {
      this.downloads = x.result
      this.pages = x.pages
      this.page = x.page
    })
  }

  onChange (evt: BvEvent, page: number): void {
    this.update(page)
  }

  onSelect (d: Download): void {
    this.selectedDownload = d
    this.selectedDownloadId = d.id
  }

  pageLinkGenerator (pageNum: number): string {
    return `/downloads/${pageNum}`
  }
}
</script>

<style scoped lang="scss">
</style>
