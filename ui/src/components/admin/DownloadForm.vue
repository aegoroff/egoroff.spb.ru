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
            <div class="mb-3" v-if="mode === 'create'">
              <label :for="`${modalId}-id-input`" class="form-label"
                >Идентификатор</label
              >
              <input
                type="number"
                class="form-control"
                :id="`${modalId}-id-input`"
                v-model.number="localDownload.id"
                required
              />
              <div class="invalid-feedback">ID обязателен</div>
            </div>
            <div class="mb-3">
              <label :for="`${modalId}-title-input`" class="form-label"
                >Название</label
              >
              <input
                type="text"
                class="form-control"
                :id="`${modalId}-title-input`"
                v-model="localDownload.title"
                required
              />
              <div class="invalid-feedback">название обязательно</div>
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
            {{ mode === 'create' ? 'Создать' : 'Сохранить' }}
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
import { Download } from "@/models/portfolio";
import { closeModalById } from "@/util";

const props = defineProps<{
  modalId: string;
  mode: "create" | "edit";
  download?: Download;
}>();

const emptyDownload = (): Download => ({ id: 0, title: "" });

const localDownload = ref<Download>(
  props.mode === "edit" && props.download
    ? { ...props.download }
    : emptyDownload()
);

watch(
  () => props.download,
  (newDownload) => {
    if (props.mode === "edit" && newDownload) {
      localDownload.value = { ...newDownload };
    }
  },
  { deep: true }
);

const modalTitle = computed(() =>
  props.mode === "create"
    ? "Создать новую загрузку"
    : localDownload.value.title
);

const onOk = async (): Promise<void> => {
  const apiService = new ApiService();
  try {
    await apiService.editDownload(localDownload.value);
    closeModalById(props.modalId);
    if (props.mode === "create") {
      emitter.emit("downloadCreated");
      localDownload.value = emptyDownload();
    } else {
      emitter.emit("downloadUpdated");
    }
  } catch (error) {
    console.error(
      props.mode === "create"
        ? "Failed to create download:"
        : "Failed to edit download:",
      error
    );
  }
};
</script>

<style scoped></style>
