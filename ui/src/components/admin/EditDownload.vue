<template>
  <b-modal size="xl" scrollable id="edit-download" :title="download.title" @ok="onOk">
    <form ref="form">
      <b-tabs>
        <b-tab title="Свойства" active>
          <b-form-group label="Название" label-for="download-title-input" invalid-feedback="название обязательно">
            <b-form-input
              id="download-title-input"
              v-model="download.title"
              required
            ></b-form-input>
          </b-form-group>
        </b-tab>
      </b-tabs>
    </form>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'

export class Download {
  public id!: number
  public title!: string
}

@Component({
  providers: [ApiService]
})
export default class EditDownload extends Vue {
  @Prop() private download!: Download
  @inject() private api!: ApiService

  onOk (): void {
    this.api.editDownload(this.download)
  }
}
</script>

<style scoped>

</style>
