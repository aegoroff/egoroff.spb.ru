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
            :disabled="busy"
          ></button>
        </div>
        <div class="modal-body">
          <form ref="form" novalidate :class="{ 'was-validated': validated }" @submit.prevent="onOk">
            <div class="mb-3" v-if="mode === 'create'">
              <label :for="`${modalId}-id-input`" class="form-label"
                >Идентификатор</label
              >
              <input
                type="number"
                class="form-control"
                :id="`${modalId}-id-input`"
                v-model.number="localDownload.id"
                min="1"
                required
              />
              <div class="invalid-feedback">ID обязателен и должен быть больше нуля</div>
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
              <div class="invalid-feedback">Название обязательно</div>
            </div>
          </form>
        </div>
        <div class="modal-footer">
          <button
            type="button"
            class="btn btn-secondary"
            data-bs-dismiss="modal"
            :disabled="busy"
          >
            Отмена
          </button>
          <button type="button" class="btn btn-primary" :disabled="busy" @click="onOk">
            <span v-if="busy" class="spinner-border spinner-border-sm me-1" role="status"></span>
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
import { useModalForm } from "@/composables/useModalForm";
import { useNotify } from "@/composables/useNotify";

const props = defineProps<{
  modalId: string;
  mode: "create" | "edit";
  download?: Download;
}>();

const emptyDownload = (): Download => ({ id: 0, title: "" });

const notify = useNotify();

const localDownload = ref<Download>(
  props.mode === "edit" && props.download
    ? { ...props.download }
    : emptyDownload()
);

// Unsaved edits are discarded on close; a create draft is kept until it is submitted.
const { form, validated, busy, submit } = useModalForm({
  modalId: props.modalId,
  onHidden: () => {
    if (props.mode === "edit" && props.download) {
      localDownload.value = { ...props.download };
    }
  },
});

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
  const saved = await submit(
    () => apiService.editDownload(localDownload.value),
    props.mode === "create"
      ? "Не удалось создать загрузку"
      : "Не удалось сохранить загрузку"
  );
  if (!saved) {
    return;
  }
  if (props.mode === "create") {
    localDownload.value = emptyDownload();
    emitter.emit("downloadCreated");
    notify.success("Загрузка создана");
  } else {
    emitter.emit("downloadUpdated");
    notify.success("Загрузка сохранена");
  }
};
</script>

<style scoped></style>
