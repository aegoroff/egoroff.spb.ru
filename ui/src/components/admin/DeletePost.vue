<template>
  <div class="modal fade" id="delete-post" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">Удалить пост</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
        </div>
        <div class="modal-body">
          <p class="my-4">Действительно удалить пост?</p>
        </div>
        <div class="modal-footer">
          <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Отмена</button>
          <button type="button" class="btn btn-danger" @click="onOk">Удалить</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'
import ApiService from '@/services/ApiService'
import { emitter } from '@/main'

export default defineComponent({
  name: 'DeletePost',
  props: {
    postId: {
      type: Number as PropType<number>,
      required: true
    }
  },
  setup(props) {
    const onOk = (): void => {
      const apiService = new ApiService()
      apiService.deletePost(props.postId)
      emitter.emit('postDeleted')
      
      // Закрываем модальное окно
      const modalElement = document.getElementById('delete-post')
      if (modalElement) {
        const modal = (window as any).bootstrap.Modal.getInstance(modalElement)
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