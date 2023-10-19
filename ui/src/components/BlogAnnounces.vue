<template>
  <dl itemscope id="blogcontainer">
    <div v-for="post in posts" :key="post.id">
      <dt><small>
        <DateFormatter v-bind:date="post.Created" format-str="LL"></DateFormatter>
      </small>&nbsp;
        <a itemprop="url" v-bind:href="'/blog/' + post.id + '.html'"><span itemprop="name">{{ post.id }}&nbsp;|&nbsp;{{ post.Title }}</span></a>
      </dt>
      <dd itemprop="description" v-html="post.ShortText">{{ post.ShortText }}</dd>
    </div>
  </dl>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import DateFormatter from '@/components/DateFomatter.vue'
import ApiService, { Query } from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'
import BlogPagination from '@/components/BlogPagination.vue'

export class Post {
  public Key!: string
  public Created!: string
  public id!: number
  public Title!: string
  public ShortText!: string
}

@Component({
  components: {
    DateFormatter
  },
  providers: [ApiService]
})
export default class BlogAnnounces extends Vue {
  @Prop() private posts!: Array<Post>
  @inject() private api!: ApiService
  @Prop() private q!: string

  mounted (): void {
    const q = this.getQuery()

    this.api.getPosts<Post>(q).then(x => {
      this.posts = x.result

      const pager = new BlogPagination({
        propsData: {
          pages: x.pages,
          page: x.page
        }
      })
      pager.$mount('#blogPager')
    })
  }

  public getQuery (): Query {
    const q = new Query()
    const parts = this.q.split('&')

    for (const part of parts) {
      const elts = part.split('=')
      Reflect.set(q, elts[0], elts[1])
    }
    return q
  }
}
</script>

<style scoped>

</style>
