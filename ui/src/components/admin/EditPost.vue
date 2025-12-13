<template>
  <div class="modal fade" id="edit-post" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog modal-xl">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ post.Title }}</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
        </div>
        <div class="modal-body">
          <form ref="form">
            <ul class="nav nav-tabs" role="tablist">
              <li class="nav-item" role="presentation">
                <button class="nav-link active" id="properties-tab-btn" data-bs-toggle="tab" 
                        data-bs-target="#properties-tab" type="button">Свойства</button>
              </li>
              <li class="nav-item" role="presentation">
                <button class="nav-link" id="text-tab-btn" data-bs-toggle="tab" 
                        data-bs-target="#text-tab" type="button">Основной текст</button>
              </li>
            </ul>
            <div class="tab-content mt-3">
              <div class="tab-pane fade show active" id="properties-tab">
                <div class="mb-3">
                  <label for="post-title-input" class="form-label">Название</label>
                  <input type="text" class="form-control" id="post-title-input" v-model="post.Title" required>
                  <div class="invalid-feedback">название обязательно</div>
                </div>
                <div class="mb-3">
                  <label for="post-tags" class="form-label">Теги</label>
                  <input type="text" class="form-control" id="post-tags" v-model="tagsString" 
                         placeholder="Введите теги через запятую, точку с запятой или пробел">
                </div>
                <div class="mb-3">
                  <label for="post-short-text" class="form-label">Краткое описание</label>
                  <textarea class="form-control" id="post-short-text" v-model="post.ShortText" 
                            rows="3" max-rows="6"></textarea>
                </div>
                <div class="form-check mb-3">
                  <input class="form-check-input" type="checkbox" id="post-public" v-model="post.IsPublic">
                  <label class="form-check-label" for="post-public">
                    Опубликовано
                  </label>
                </div>
                <div class="form-check">
                  <input class="form-check-input" type="checkbox" id="post-markdown" v-model="post.Markdown">
                  <label class="form-check-label" for="post-markdown">
                    Markdown
                  </label>
                </div>
              </div>
              <div class="tab-pane fade" id="text-tab">
                <div class="mb-3">
                  <label for="post-text" class="form-label">Основной текст</label>
                  <textarea class="form-control" id="post-text" v-model="post.Text" rows="20"></textarea>
                </div>
              </div>
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
import { defineComponent, PropType, computed } from 'vue'
import ApiService from '@/services/ApiService'

export class Post {
  public Created!: string
  public id!: number
  public Title!: string
  public IsPublic!: boolean
  public Markdown!: boolean
  public Tags!: Array<string>
  public Text!: string
  public ShortText!: string
}

export default defineComponent({
  name: 'EditPost',
  props: {
    post: {
      type: Object as PropType<Post>,
      required: true
    }
  },
  setup(props) {
    const tagsString = computed({
      get: () => props.post.Tags.join(', '),
      set: (value: string) => {
        props.post.Tags = value.split(/[,;\s]+/).filter(tag => tag.trim())
      }
    })
    
    const onOk = (): void => {
      const apiService = new ApiService()
      apiService.editPost(props.post)
      
      // Закрываем модальное окно
      const modalElement = document.getElementById('edit-post')
      if (modalElement) {
        const modal = bootstrap.Modal.getInstance(modalElement)
        if (modal) {
          modal.hide()
        }
      }
    }
    
    return {
      tagsString,
      onOk
    }
  }
})
</script>

<style scoped>

</style>