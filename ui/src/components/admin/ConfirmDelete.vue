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
            :disabled="busy"
          ></button>
        </div>
        <div class="modal-body">
          <p class="my-4">
            {{ message }}
            <strong v-if="itemTitle">«{{ itemTitle }}»</strong>
            <span class="text-body-secondary"> (ID {{ itemId }})</span>
          </p>
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
          <button type="button" class="btn btn-danger" :disabled="busy" @click="onOk">
            <span v-if="busy" class="spinner-border spinner-border-sm me-1" role="status"></span>
            Удалить
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, ref } from "vue";
import ApiService from "@/services/ApiService";
import { emitter } from "@/events";
import { closeModalById } from "@/util";
import { useNotify } from "@/composables/useNotify";

const props = defineProps<{
  modalId: string;
  title: string;
  message: string;
  itemId: number;
  itemTitle?: string;
  kind: "post" | "download";
}>();

const apiService = new ApiService();

const actions = {
  post: {
    remove: (id: number) => apiService.deletePost(id),
    emit: () => emitter.emit("postDeleted"),
    success: "Пост удалён",
    failure: "Не удалось удалить пост",
  },
  download: {
    remove: (id: number) => apiService.deleteDownload(id),
    emit: () => emitter.emit("downloadDeleted"),
    success: "Загрузка удалена",
    failure: "Не удалось удалить загрузку",
  },
};

const notify = useNotify();
const busy = ref(false);

const onOk = async (): Promise<void> => {
  const action = actions[props.kind];
  busy.value = true;
  try {
    await action.remove(props.itemId);
    action.emit();
    notify.success(action.success);
  } catch (error) {
    notify.error(action.failure, error);
    return;
  } finally {
    busy.value = false;
  }
  await nextTick();
  closeModalById(props.modalId);
};
</script>

<style scoped></style>
