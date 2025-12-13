<template>
  <div class="modal fade" id="edit-download" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog modal-xl">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ download.title }}</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
        </div>
        <div class="modal-body">
          <form ref="form">
            <div class="mb-3">
              <label for="download-title-input" class="form-label">Название</label>
              <input type="text" class="form-control" id="download-title-input" v-model="download.title" required>
              <div class="invalid-feedback">название обязательно</div>
            </div>
          </form>
        </div>
        <div class="modal-footer">
          <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Отмена</button>
          <button type="button" class="btn btn-primary" @click="onOk">Сохранить</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'
import ApiService from '@/services/ApiService.vue'

export class Download {
  public id!: number
  public title!: string
}

export default defineComponent({
  name: 'EditDownload',
  props: {
    download: {
      type: Object as PropType<Download>,
      required: true
    }
  },
  setup(props) {
    const onOk = (): void => {
      const apiService = new ApiService()
      apiService.editDownload(props.download)
      
      // Закрываем модальное окно
      const modalElement = document.getElementById('edit-download')
      if (modalElement) {
        const modal = bootstrap.Modal.getInstance(modalElement)
        if (modal) {
          modal.hide()
        }
      }
    }
    
    return {
      onOk
    }
  }
})
</script>

<style scoped>

</style>