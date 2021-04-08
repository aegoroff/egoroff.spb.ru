<template>
  <dl itemscope id="blogcontainer">
    <div v-for="post in posts" :key="post.id">
      <dt><small><DateFormatter v-bind:date="post.Created" format-str="LL"></DateFormatter> {{ post.Created }}</small>&nbsp;
        <a itemprop="url" href="/blog/{{ post.id }}.html"><span itemprop="name">{{ post.Title }}</span></a>
      </dt>
      <dd itemprop="description" v-html="post.ShortText">{{ post.ShortText }}</dd>
    </div>
  </dl>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import DateFormatter from '@/components/DateFomatter.vue'

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
  }
})
export default class BlogAnnounces extends Vue {
  @Prop() private posts!: Array<Post>
}
</script>

<style scoped>

</style>
