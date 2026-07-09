<template>
  <div class="modal fade" id="edit-download" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog modal-xl">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ localDownload.title }}</h5>
          <button
            type="button"
            class="btn-close"
            data-bs-dismiss="modal"
          ></button>
        </div>
        <div class="modal-body">
          <form ref="form">
            <div class="mb-3">
              <label for="download-title-input" class="form-label"
                >Название</label
              >
              <input
                type="text"
                class="form-control"
                id="download-title-input"
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
            Сохранить
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ApiService from "@/services/ApiService";
import { closeModalById } from "@/util";
import { Download } from '@/models/portfolio'
import { ref, watch } from 'vue'

const props = defineProps<{
  download: Download
}>()

const localDownload = ref<Download>({ ...props.download })

watch(() => props.download, (newDownload) => {
  localDownload.value = { ...newDownload }
}, { deep: true })

const onOk = async (): Promise<void> => {
  const apiService = new ApiService();
  try {
    await apiService.editDownload(localDownload.value);
    closeModalById("edit-download");
  } catch (error) {
    console.error("Failed to edit download:", error);
  }
};
</script>

<style scoped></style>