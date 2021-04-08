<template>
  <dl itemscope id="blogcontainer">
    <div v-for="post in posts" :key="post.id">
      <dt><small><DateFormatter v-bind:date="post.Created" format-str="LL"></DateFormatter></small>&nbsp;
        <a itemprop="url" v-bind:href="'/blog/' + post.id + '.html'"><span itemprop="name">{{ post.Title }}</span></a>
      </dt>
      <dd itemprop="description" v-html="post.ShortText">{{ post.ShortText }}</dd>
    </div>
  </dl>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import DateFormatter from '@/components/DateFomatter.vue'
import ApiService from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'

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

  mounted (): void {
    this.api.getPosts().then(x => {
      this.posts = x.result
      console.log(this.posts)
    })
  }
}
</script>

<style scoped>

</style>
