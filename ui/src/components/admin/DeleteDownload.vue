<template>
  <b-modal scrollable id="delete-download" title="Удалить загрузку" @ok="onOk">
    <p class="my-4">Действительно удалить загрузку?</p>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'
import { bus } from '@/main'

@Component({
  providers: [ApiService]
})
export default class DeleteDownload extends Vue {
  @Prop() private downloadId!: number
  @inject() private api!: ApiService

  onOk (): void {
    this.api.deleteDownload(this.downloadId)
    bus.$emit('downloadDeleted')
  }
}
</script>

<style scoped>

</style>
