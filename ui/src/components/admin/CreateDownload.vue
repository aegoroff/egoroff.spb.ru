<template>
  <b-modal size="xl" scrollable id="create-download" title="Создать новую загрузку" @ok="onOk">
    <form ref="form">
      <b-form-group label="Идентификатор" label-for="create-download-title-input" invalid-feedback="ID обязателен">
        <b-form-input
          id="create-download-id-input"
          v-model.number="download.id"
          type="number"
          number
          required
        ></b-form-input>
      </b-form-group>
      <b-form-group label="Название" label-for="create-download-title-input" invalid-feedback="название обязательно">
        <b-form-input
          id="create-download-title-input"
          v-model="download.title"
          required
        ></b-form-input>
      </b-form-group>
    </form>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'
import { bus } from '@/main'
import { Download } from './EditDownload.vue'

@Component({
  providers: [ApiService]
})
export default class CreateDownload extends Vue {
  @Prop({ default: new Download() }) public download!: Download;
  @inject() private api!: ApiService;

  onOk (): void {
    this.api.editDownload(this.download)
    bus.$emit('downloadCreated')
  }
}
</script>

<style scoped>

</style>
