<template>
  <div class="modal fade" id="create-post" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog modal-xl">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">Создать новый пост</h5>
          <button
            type="button"
            class="btn-close"
            data-bs-dismiss="modal"
          ></button>
        </div>
        <div class="modal-body">
          <form ref="form">
            <ul class="nav nav-tabs" role="presentation">
              <li class="nav-item" role="presentation">
                <button
                  class="nav-link active"
                  id="create-properties-tab-btn"
                  data-bs-toggle="tab"
                  data-bs-target="#create-properties-tab"
                  type="button"
                >
                  Свойства
                </button>
              </li>
              <li class="nav-item" role="presentation">
                <button
                  class="nav-link"
                  id="create-text-tab-btn"
                  data-bs-toggle="tab"
                  data-bs-target="#create-text-tab"
                  type="button"
                >
                  Основной текст
                </button>
              </li>
            </ul>
            <div class="tab-content mt-3">
              <div class="tab-pane fade show active" id="create-properties-tab">
                <div class="mb-3">
                  <label for="create-post-title-input" class="form-label"
                    >Название</label
                  >
                  <input
                    type="text"
                    class="form-control"
                    id="create-post-title-input"
                    v-model="post.Title"
                    required
                  />
                  <div class="invalid-feedback">название обязательно</div>
                </div>
                <div class="mb-3">
                  <label for="create-post-created" class="form-label">Дата создания</label>
                  <input
                    type="datetime-local"
                    class="form-control"
                    id="create-post-created"
                    v-model="post.Created"
                    required
                  />
                  <div class="invalid-feedback">дата создания обязательна</div>
                </div>
                <div class="mb-3">
                  <label for="create-post-tags" class="form-label">Теги</label>
                  <input
                    type="text"
                    class="form-control"
                    id="create-post-tags"
                    v-model="tagsString"
                    placeholder="Введите теги через запятую, точку с запятой или пробел"
                  />
                </div>
                <div class="mb-3">
                  <label for="create-post-short-text" class="form-label"
                    >Краткое описание</label
                  >
                  <textarea
                    class="form-control"
                    id="create-post-short-text"
                    v-model="post.ShortText"
                    rows="3"
                    max-rows="6"
                  ></textarea>
                </div>
                <div class="form-check mb-3">
                  <input
                    class="form-check-input"
                    type="checkbox"
                    id="create-post-public"
                    v-model="post.IsPublic"
                  />
                  <label class="form-check-label" for="create-post-public">
                    Опубликовано
                  </label>
                </div>
                <div class="form-check">
                  <input
                    class="form-check-input"
                    type="checkbox"
                    id="create-post-markdown"
                    v-model="post.Markdown"
                  />
                  <label class="form-check-label" for="create-post-markdown">
                    Markdown
                  </label>
                </div>
              </div>
              <div class="tab-pane fade" id="create-text-tab">
                <div class="mb-3">
                  <label for="create-post-text" class="form-label"
                    >Основной текст</label
                  >
                  <textarea
                    class="form-control"
                    id="create-post-text"
                    v-model="post.Text"
                    rows="20"
                  ></textarea>
                </div>
              </div>
            </div>
          </form>
        </div>
        <div class="modal-footer">
          <button
            type="button"
            class="btn btn-secondary"
            data-bs-dismiss="modal"
          >
            Отмена
          </button>
          <button type="button" class="btn btn-primary" @click="onOk">
            Создать
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import ApiService from "@/services/ApiService";
import { emitter } from "@/main";
import { EditablePost } from "@/models/blog";
import { closeModalById } from "@/util";

const post = ref<EditablePost>({
  Created: "",
  Modified: "",
  id: 0,
  Title: "",
  IsPublic: false,
  Markdown: false,
  Tags: [],
  Text: "",
  ShortText: "",
});

const tagsString = computed({
  get: () => post.value.Tags.join(", "),
  set: (value: string) => {
    post.value.Tags = value.split(/[,;\s]+/).filter((tag) => tag.trim());
  },
});

const formatDateTime = (dateTime: string): string => {
  if (!dateTime) {
    const now = new Date();
    return now.toISOString().replace(/\.\d{3}Z$/, 'Z');
  }
  // datetime-local возвращает формат "2024-04-25T04:52"
  // Добавляем секунды и Z если их нет
  if (dateTime.endsWith('Z')) {
    return dateTime;
  }
  if (dateTime.length === 16) {
    // Формат "2024-04-25T04:52" - добавляем секунды
    return `${dateTime}:00Z`;
  }
  if (dateTime.length === 19) {
    // Формат "2024-04-25T04:52:37" - добавляем Z
    return `${dateTime}Z`;
  }
  return dateTime;
};

const onOk = async (): Promise<void> => {
  const apiService = new ApiService();
  try {
    const formattedPost = {
      ...post.value,
      Created: formatDateTime(post.value.Created),
      Modified: formatDateTime(post.value.Created),
    };
    await apiService.createPost(formattedPost);
    emitter.emit("postCreated");
    closeModalById("create-post");

    // Сброс формы
    post.value = {
      Created: "",
      Modified: "",
      id: 0,
      Title: "",
      IsPublic: false,
      Markdown: false,
      Tags: [],
      Text: "",
      ShortText: "",
    };
  } catch (error) {
    console.error("Failed to create post:", error);
  }
};
</script>

<style scoped></style>