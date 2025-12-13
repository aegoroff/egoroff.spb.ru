<template>
  <div class="tags">
    <ul class="list-inline">
      <li class="list-inline-item" v-for="tag in tags" :key="tag.title">
        <a
          :href="`/blog/#tag=${tag.title}`"
          :class="[currentTag === tag.title ? `btn btn-outline-dark ${tagClass(tag.level)}` : tagClass(tag.level)]"
          @click.prevent="update(tag.title, 1)"
          :id="`t_${tag.title}`">{{ tag.title }}</a>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType, ref, onMounted } from 'vue'
import { emitter } from '@/main'

export class Tag {
  public title!: string
  public level!: number
}

export default defineComponent({
  name: 'Tags',
  props: {
    tags: {
      type: Array as PropType<Tag[]>,
      required: true
    }
  },
  setup(props) {
    const currentTag = ref('')
    const tagsClasses: Record<number, string> = {
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
    }

    onMounted(() => {
      emitter.on('pageChanged', (data: any) => {
        const page = typeof data === 'number' ? data : 1
        const parts = window.location.hash.substring(1).split('&')

        for (const part of parts) {
          const elts = part.split('=')
          if (elts[0] === 'tag') {
            update(elts[1], page)
            return
          }
        }
      })

      emitter.on('dateSelectionChanged', () => {
        currentTag.value = ''
      })
    })

    const tagClass = (ix: number): string => {
      return tagsClasses[ix]
    }

    const update = (tag: string, page: number): void => {
      emitter.emit('tagChanged', tag)
      currentTag.value = tag
      const blogContainer = document.getElementById('blogcontainer')
      const blogTitle = document.getElementById('blogSmallTitle')
      
      if (blogContainer) {
        blogContainer.innerHTML = `<blog-announces q="tag=${tag}&page=${page}"></blog-announces>`
      }
      
      if (blogTitle) {
        blogTitle.innerHTML = `<blog-title text="все посты по метке: ${tag}"></blog-title>`
      }
    }

    return {
      currentTag,
      tagClass,
      update
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