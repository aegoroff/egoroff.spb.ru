<template>
  <div class="modal fade" id="create-download" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog modal-xl">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">Создать новую загрузку</h5>
          <button
            type="button"
            class="btn-close"
            data-bs-dismiss="modal"
          ></button>
        </div>
        <div class="modal-body">
          <form ref="form">
            <div class="mb-3">
              <label for="create-download-id-input" class="form-label"
                >Идентификатор</label
              >
              <input
                type="number"
                class="form-control"
                id="create-download-id-input"
                v-model.number="download.id"
                required
              />
              <div class="invalid-feedback">ID обязателен</div>
            </div>
            <div class="mb-3">
              <label for="create-download-title-input" class="form-label"
                >Название</label
              >
              <input
                type="text"
                class="form-control"
                id="create-download-title-input"
                v-model="download.title"
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
            Создать
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import ApiService from "@/services/ApiService";
import { emitter } from "@/main";
import { Download } from "./EditDownload.vue";
import { closeModalById } from "@/util";

const download = ref<Download>({
  id: 0,
  title: "",
});

const onOk = (): void => {
  const apiService = new ApiService();
  apiService.editDownload(download.value);
  emitter.emit("downloadCreated");

  closeModalById("create-download");

  // Сброс формы
  download.value = { id: 0, title: "" };
};
</script>

<style scoped></style>