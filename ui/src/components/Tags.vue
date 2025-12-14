<template>
  <div class="tags">
    <ul class="list-inline">
      <li class="list-inline-item" v-for="tag in tags" :key="tag.title">
        <a
          :href="`/blog/#tag=${tag.title}`"
          :class="[currentTag === tag.title ? `btn btn-outline-dark ${tagClass(tag.level)}` : `btn ${tagClass(tag.level)}`]"
          v-on:click="update(tag.title, 1)"
          :id="`t_${tag.title}`">{{ tag.title }}</a>
      </li>
    </ul>
  </div>
</template>
<script lang="ts">
import { defineComponent, createApp } from 'vue'
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import { emitter } from '@/main'

export class Tag {
  public title!: string
  public level!: number
}

export default defineComponent({
  name: 'Tags',

  props: {
    tags: {
      type: Array<Tag>,
      required: true
    },
    currentTag: {
      type: String,
      required: true
    }
  },

  data() {
    return {
      tagsClasses: {
        0: 'tagRank10',
        1: 'tagRank9',
        2: 'tagRank8',
        3: 'tagRank7',
        4: 'tagRank6',
        5: 'tagRank5',
        6: 'tagRank4',
        7: 'tagRank3',
        8: 'tagRank2',
        10: 'tagRank1'
      } as Record<number, string>
    }
  },

  created() {
    emitter.on('pageChanged', (page) => {
      const parts = window.location.hash.substring(1).split('&')

      for (const part of parts) {
        const [key, value] = part.split('=')
        if (key === 'tag') {
          this.update(value, page)
          return
        }
      }
    })

    emitter.on('dateSelectionChanged', () => {
      this.$emit('update:currentTag', '')
    })
  },

  methods: {
    tagClass(ix: number): string {
      return this.tagsClasses[ix]
    },

    update(tag: string, page: number): void {
      emitter.emit('tagChanged' as any, tag)
      this.$emit('update:currentTag', tag)

      createApp(BlogAnnounces, {
        q: `tag=${tag}&page=${page}`
      }).mount('#blogcontainer')

      createApp(BlogTitle, {
        text: `все посты по метке: ${tag}`
      }).mount('#blogSmallTitle')
    }
  }
})
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