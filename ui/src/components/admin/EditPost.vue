<template>
  <b-modal size="xl" scrollable id="edit-post" :title="post.Title" @ok="onOk">
    <form ref="form">
      <b-form-group label="Название" label-for="post-title-input" invalid-feedback="название обязательно">
        <b-form-input
          id="post-title-input"
          v-model="post.Title"
          required
        ></b-form-input>
      </b-form-group>

      <b-form-group label="Теги" label-for="post-tags">
        <b-form-tags
          input-id="post-tags"
          v-model="post.Tags"
          separator=" ,;"
          no-add-on-enter
        ></b-form-tags>
      </b-form-group>
      <b-form-group label="Краткое описание" label-for="post-short-text">
        <b-form-textarea
          id="post-short-text"
          v-model="post.ShortText"
          rows="3"
          max-rows="6"
        ></b-form-textarea>
      </b-form-group>

      <b-form-checkbox id="post-public" v-model="post.IsPublic">
        Опубликовано
      </b-form-checkbox>
      <b-form-checkbox id="post-markdown" v-model="post.Markdown">
        Markdown
      </b-form-checkbox>
    </form>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'

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

@Component({
  providers: [ApiService]
})
export default class EditPost extends Vue {
  @Prop() private post!: Post
  @inject() private api!: ApiService

  onOk (): void {
    this.api.editPost(this.post)
  }
}
</script>

<style scoped>

</style>
