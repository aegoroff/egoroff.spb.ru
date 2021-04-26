<template>
  <b-modal scrollable id="delete-post" title="Удалить пост" @ok="onOk">
    <p class="my-4">Действительно удалить пост?</p>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'

@Component({
  providers: [ApiService]
})
export default class DeletePost extends Vue {
  @Prop() private postId!: number
  @inject() private api!: ApiService

  onOk (): void {
    this.api.deletePost(this.postId)
  }
}
</script>

<style scoped>

</style>
