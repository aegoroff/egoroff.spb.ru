<template>
  <div class="modal fade" :id="modalId" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog modal-xl">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ modalTitle }}</h5>
          <button
            type="button"
            class="btn-close"
            data-bs-dismiss="modal"
          ></button>
        </div>
        <div class="modal-body">
          <form>
            <ul class="nav nav-tabs" role="presentation">
              <li class="nav-item" role="presentation">
                <button
                  class="nav-link active"
                  :id="ids.propertiesTabBtn"
                  data-bs-toggle="tab"
                  :data-bs-target="`#${ids.propertiesTab}`"
                  type="button"
                >
                  Свойства
                </button>
              </li>
              <li class="nav-item" role="presentation">
                <button
                  class="nav-link"
                  :id="ids.textTabBtn"
                  data-bs-toggle="tab"
                  :data-bs-target="`#${ids.textTab}`"
                  type="button"
                >
                  Основной текст
                </button>
              </li>
            </ul>
            <div class="tab-content mt-3">
              <div
                class="tab-pane fade show active"
                :id="ids.propertiesTab"
              >
                <div class="mb-3">
                  <label :for="ids.title" class="form-label">Название</label>
                  <input
                    type="text"
                    class="form-control"
                    :id="ids.title"
                    v-model="localPost.Title"
                    required
                  />
                  <div class="invalid-feedback">название обязательно</div>
                </div>
                <div class="mb-3" v-if="mode === 'create'">
                  <label :for="ids.created" class="form-label"
                    >Дата создания</label
                  >
                  <input
                    type="datetime-local"
                    class="form-control"
                    :id="ids.created"
                    v-model="localPost.Created"
                    required
                  />
                  <div class="invalid-feedback">дата создания обязательна</div>
                </div>
                <div class="mb-3">
                  <label :for="ids.tags" class="form-label">Теги</label>
                  <input
                    type="text"
                    class="form-control"
                    :id="ids.tags"
                    v-model="tagsString"
                    placeholder="Введите теги через запятую, точку с запятой или пробел"
                  />
                </div>
                <div class="mb-3">
                  <label :for="ids.shortText" class="form-label"
                    >Краткое описание</label
                  >
                  <textarea
                    class="form-control"
                    :id="ids.shortText"
                    v-model="localPost.ShortText"
                    rows="3"
                    max-rows="6"
                  ></textarea>
                </div>
                <div class="form-check mb-3">
                  <input
                    class="form-check-input"
                    type="checkbox"
                    :id="ids.isPublic"
                    v-model="localPost.IsPublic"
                  />
                  <label class="form-check-label" :for="ids.isPublic">
                    Опубликовано
                  </label>
                </div>
                <div class="form-check">
                  <input
                    class="form-check-input"
                    type="checkbox"
                    :id="ids.markdown"
                    v-model="localPost.Markdown"
                  />
                  <label class="form-check-label" :for="ids.markdown">
                    Markdown
                  </label>
                </div>
              </div>
              <div class="tab-pane fade" :id="ids.textTab">
                <div class="mb-3">
                  <label :for="ids.text" class="form-label"
                    >Основной текст</label
                  >
                  <textarea
                    class="form-control"
                    :id="ids.text"
                    v-model="localPost.Text"
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
            {{ mode === "create" ? "Создать" : "Сохранить" }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import ApiService from "@/services/ApiService";
import { emitter } from "@/events";
import { EditablePost } from "@/models/blog";
import { closeModalById } from "@/util";

const props = defineProps<{
  modalId: string;
  mode: "create" | "edit";
  post?: EditablePost;
}>();

const emptyPost = (): EditablePost => ({
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

const localPost = ref<EditablePost>(
  props.mode === "edit" && props.post ? { ...props.post } : emptyPost()
);

watch(
  () => props.post,
  (newPost) => {
    if (props.mode === "edit" && newPost) {
      localPost.value = { ...newPost };
    }
  },
  { deep: true }
);

/** Preserve original DOM ids from CreatePost / EditPost. */
const ids = computed(() =>
  props.mode === "create"
    ? {
        propertiesTabBtn: "create-properties-tab-btn",
        propertiesTab: "create-properties-tab",
        textTabBtn: "create-text-tab-btn",
        textTab: "create-text-tab",
        title: "create-post-title-input",
        created: "create-post-created",
        tags: "create-post-tags",
        shortText: "create-post-short-text",
        isPublic: "create-post-public",
        markdown: "create-post-markdown",
        text: "create-post-text",
      }
    : {
        propertiesTabBtn: "properties-tab-btn",
        propertiesTab: "properties-tab",
        textTabBtn: "text-tab-btn",
        textTab: "text-tab",
        title: "post-title-input",
        created: "",
        tags: "post-tags",
        shortText: "post-short-text",
        isPublic: "post-public",
        markdown: "post-markdown",
        text: "post-text",
      }
);

const modalTitle = computed(() =>
  props.mode === "create" ? "Создать новый пост" : localPost.value.Title
);

const tagsString = computed({
  get: () => localPost.value.Tags.join(", "),
  set: (value: string) => {
    localPost.value.Tags = value.split(/[,;\s]+/).filter((tag) => tag.trim());
  },
});

const formatDateTime = (dateTime: string): string => {
  if (!dateTime) {
    const now = new Date();
    return now.toISOString().replace(/\.\d{3}Z$/, "Z");
  }
  // datetime-local returns "2024-04-25T04:52" format
  if (dateTime.endsWith("Z")) {
    return dateTime;
  }
  if (dateTime.length === 16) {
    return `${dateTime}:00Z`;
  }
  if (dateTime.length === 19) {
    return `${dateTime}Z`;
  }
  return dateTime;
};

const onOk = async (): Promise<void> => {
  const apiService = new ApiService();
  try {
    if (props.mode === "create") {
      const formattedPost = {
        ...localPost.value,
        Created: formatDateTime(localPost.value.Created),
        Modified: formatDateTime(localPost.value.Created),
      };
      await apiService.createPost(formattedPost);
      emitter.emit("postCreated");
      closeModalById(props.modalId);
      localPost.value = emptyPost();
    } else {
      await apiService.editPost(localPost.value);
      emitter.emit("postUpdated");
      closeModalById(props.modalId);
    }
  } catch (error) {
    console.error(
      props.mode === "create" ? "Failed to create post:" : "Failed to edit post:",
      error
    );
  }
};
</script>

<style scoped></style>
