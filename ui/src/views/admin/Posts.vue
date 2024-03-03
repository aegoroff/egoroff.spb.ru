<template>
  <div>
    <b-pagination-nav
      id="posts-pager"
      aria-controls="posts-table"
      v-if="pages > 1"
      v-bind:number-of-pages="pages"
      v-model="page"
      @page-click="onChange"
      align="center"
      :link-gen="pageLinkGenerator"
      use-router
    ></b-pagination-nav>
    <EditPost id="edit-post" :post="selectedPost"></EditPost>
    <DeletePost id="delete-post" :postId="selectedPostId"></DeletePost>
    <b-table-lite responsive small striped hover :items="posts" :fields="fields" id="posts-table">
      <template #cell(Created)="data">
        <date-formatter :date="data.value" format-str="L"></date-formatter>
      </template>
      <template #cell(Title)="data">
        <a v-b-modal.edit-post href="#" @click="onSelect(data.item)">{{ data.value }}</a>
      </template>
      <template #cell(-)="data">
        <a v-b-modal.delete-post href="#" @click="onSelect(data.item)">
          <AppIcon icon="trash-alt"></AppIcon>
        </a>
      </template>
    </b-table-lite>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService, { Query } from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'
import DateFormatter from '@/components/DateFomatter.vue'
import { BvEvent } from 'bootstrap-vue'
import EditPost, { Post } from '@/components/admin/EditPost.vue'
import DeletePost from '@/components/admin/DeletePost.vue'
import AppIcon from '@/components/AppIcon.vue'
import { bus } from '@/main'

@Component({
  components: {
    DateFormatter,
    EditPost,
    DeletePost,
    AppIcon
  },
  providers: [ApiService]
})
export default class Posts extends Vue {
  @inject() private api!: ApiService
  @Prop() private posts!: Array<Post>
  @Prop() private page!: number
  @Prop() private pages!: number
  @Prop() private selectedPost!: Post
  @Prop() private selectedPostId!: number
  private fields = ['-', 'id', 'Created', 'Title', 'IsPublic']

  constructor () {
    super()
    this.update(1)
    bus.$on('postDeleted', () => {
      this.update(this.page)
    })
  }

  update (page: number): void {
    const q = new Query()
    q.page = page.toString()
    q.limit = '10'
    this.api.getAdminPosts<Post>(q).then(x => {
      this.posts = x.result
      this.pages = x.pages
      this.page = x.page
    })
  }

  onChange (evt: BvEvent, page: number): void {
    this.update(page)
  }

  onSelect (p: Post): void {
    this.selectedPost = p
    this.selectedPostId = p.id
  }

  pageLinkGenerator (pageNum: number): string {
    return `/${pageNum}`
  }
}
</script>

<style scoped lang="scss">
.shortDate {
  font-weight: normal;
  color: black;
}
</style>
