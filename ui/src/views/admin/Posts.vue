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
    <b-table-lite responsive small striped hover :items="posts" :fields="fields" id="posts-table">
      <template #cell(Created)="data">
        <date-formatter :date="data.value" format-str="L"></date-formatter>
      </template>
      <template #cell(Title)="data">
        <a v-b-modal.edit-post href="#" @click="onEdit(data.item)">{{ data.value }}</a>
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

@Component({
  components: {
    DateFormatter,
    EditPost
  },
  providers: [ApiService]
})
export default class Posts extends Vue {
  @inject() private api!: ApiService
  @Prop() private posts!: Array<Post>
  @Prop() private fields!: Array<string>
  @Prop() private page!: number
  @Prop() private pages!: number
  @Prop() private selectedPost!: Post

  constructor () {
    super()
    this.update(1)
  }

  update (page: number): void {
    const q = new Query()
    q.page = page.toString()
    q.limit = '10'
    this.api.getAdminPosts<Post>(q).then(x => {
      this.fields = ['id', 'Created', 'Title', 'IsPublic']
      this.posts = x.result
      this.pages = x.pages
      this.page = x.page
    })
  }

  onChange (evt: BvEvent, page: number): void {
    this.update(page)
  }

  onEdit (p: Post): void {
    this.selectedPost = p
  }

  pageLinkGenerator (pageNum: number): string {
    return `/posts/${pageNum}`
  }
}
</script>

<style scoped lang="scss">
.shortDate {
  font-weight: normal;
  color: black;
}
</style>
