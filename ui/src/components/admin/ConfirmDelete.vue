<template>
  <div class="modal fade" :id="modalId" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ title }}</h5>
          <button
            type="button"
            class="btn-close"
            data-bs-dismiss="modal"
          ></button>
        </div>
        <div class="modal-body">
          <p class="my-4">{{ message }}</p>
        </div>
        <div class="modal-footer">
          <button
            type="button"
            class="btn btn-secondary"
            data-bs-dismiss="modal"
          >
            Отмена
          </button>
          <button type="button" class="btn btn-danger" @click="onOk">
            Удалить
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ApiService from "@/services/ApiService";
import { emitter } from "@/events";
import { closeModalById } from "@/util";

const props = defineProps<{
  modalId: string;
  title: string;
  message: string;
  itemId: number;
  kind: "post" | "download";
}>();

const onOk = async (): Promise<void> => {
  const apiService = new ApiService();
  try {
    if (props.kind === "post") {
      await apiService.deletePost(props.itemId);
      emitter.emit("postDeleted");
    } else {
      await apiService.deleteDownload(props.itemId);
      emitter.emit("downloadDeleted");
    }
    closeModalById(props.modalId);
  } catch (error) {
    console.error(
      props.kind === "post"
        ? "Failed to delete post:"
        : "Failed to delete download:",
      error
    );
  }
};
</script>

<style scoped></style>
