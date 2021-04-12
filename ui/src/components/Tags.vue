<template>
  <div class="tags">
    <ul class="list-inline">
      <li class="list-inline-item" v-for="tag in tags" :key="tag.title">
        <a
          v-bind:href="`/blog/#tag=${tag.title}`"
          v-bind:class="[currentTag === tag.title ? `btn btn-outline-dark ${tag.level}` : tag.level]"
          v-on:click="update(tag.title, 1)"
          v-bind:id="`t_${tag.title}`">{{ tag.title }}</a>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import { bus } from '@/main'

export class Tag {
  public title!: string
  public level!: string
}

@Component
export default class Tags extends Vue {
  @Prop() private tags!: Array<Tag>
  @Prop() private currentTag!: string

  created (): void {
    bus.$on('pageChanged', (data: number) => {
      const parts = window.location.hash.substr(1).split('&')

      for (const part of parts) {
        const elts = part.split('=')
        if (elts[0] === 'tag') {
          this.update(elts[1], data)
          return
        }
      }
    })

    bus.$on('dateSelectionChanged', () => {
      this.currentTag = ''
    })
  }

  update (tag: string, page: number): void {
    bus.$emit('tagChanged', tag)
    this.currentTag = tag
    const ba = new BlogAnnounces({
      propsData: {
        q: `tag=${tag}&page=${page}`
      }
    })
    ba.$mount('#blogcontainer')

    const bt = new BlogTitle({
      propsData: {
        text: `все посты по метке: ${tag}`
      }
    })
    bt.$mount('#blogSmallTitle')
  }
}

</script>

<style scoped lang="scss">
.tags {
  overflow: hidden;
  margin: 10px auto 30px;
  padding: 20px;
  line-height: 1.6em;

  span {
    font-weight: bold;
    font-style: italic;
  }

  a, span {
    position: relative;
    top: 0;
    left: -10px;
    z-index: 10;
    margin: 0;
    padding: 0 5px;
    color: #000;
    font-size: 1em;
    text-decoration: none;
  }

  a.tagRank1, span.tagRank1 {
    font-size: 2.8em;
  }

  a.tagRank2, span.tagRank2 {
    font-size: 2.6em;
    color: #222;
  }

  a.tagRank3, span.tagRank3 {
    font-size: 2.4em;
    color: #333;
  }

  a.tagRank4, span.tagRank4 {
    font-size: 2.2em;
    color: #444;
  }

  a.tagRank5, span.tagRank5 {
    font-size: 2em;
    color: #555;
  }

  a.tagRank6, span.tagRank6 {
    font-size: 1.8em;
    color: #666;
  }

  a.tagRank7, span.tagRank7 {
    font-size: 1.6em;
    color: #777;
  }

  a.tagRank8, span.tagRank8 {
    font-size: 1.4em;
    color: #888;
  }

  a.tagRank9, span.tagRank9 {
    font-size: 1.2em;
    color: #999;
  }

  a.tagRank10, span.tagRank10 {
    font-size: 1em;
    color: #AAA;
  }

  a:hover {
    text-decoration: underline;
    position: relative;
    z-index: 50;
    color: #F00;
  }
}
</style>
