<template>
  <div class="tags mt-3">
    <div class="d-flex flex-wrap gap-2">
      <a
        v-for="tag in tags"
        :key="tag.title"
        :href="`/blog/#tag=${encodeURIComponent(tag.title)}`"
        :class="[
          currentTag === tag.title
            ? `btn btn-dark btn-sm ${tagClass(tag.level)}`
            : `btn btn-sm ${tagClass(tag.level)}`,
        ]"
        @click.prevent="update(tag.title, 1)"
        :id="`t_${tag.title}`"
        >{{ tag.title }}</a
      >
    </div>
  </div>
</template>
<script lang="ts">
import { defineComponent, PropType, ref, onMounted } from "vue";
import { emitter } from "@/main";

export class Tag {
  public title!: string;
  public level!: number;
}
export default defineComponent({
  name: "Tags",
  props: {
    tags: {
      type: Array as PropType<Tag[]>,
      required: true,
    },
  },
  setup(props) {
    const currentTag = ref("");
    const tagsClasses: Record<number, string> = {
      0: "tagRank10",
      1: "tagRank9",
      2: "tagRank8",
      3: "tagRank7",
      4: "tagRank6",
      5: "tagRank5",
      6: "tagRank4",
      7: "tagRank3",
      8: "tagRank2",
      10: "tagRank1",
    };
    
    onMounted(() => {
      updateFromHash();
      
      window.addEventListener('hashchange', updateFromHash);
    });
    
    const updateFromHash = (): void => {
      const hash = window.location.hash.substring(1);
      const params = new URLSearchParams(hash);
      const tag = params.get('tag');
      
      if (tag) {
        currentTag.value = decodeURIComponent(tag);
      } else {
        currentTag.value = "";
      }
    };
    
    const tagClass = (ix: number): string => {
      return tagsClasses[ix] || "tagRank10";
    };
    
    const update = (tag: string, page: number): void => {
      const params = new URLSearchParams();
      params.set('tag', encodeURIComponent(tag));
      if (page > 1) {
        params.set('page', page.toString());
      }
      
      window.location.hash = '#' + params.toString();
    };
    
    return {
      currentTag,
      tagClass,
      update,
    };
  }
});
</script>
<style scoped lang="scss">
.tags {
  overflow: hidden;
  margin: 10px auto 30px;
  padding: 20px;
  line-height: 1.6em;
  
  .tagRank1 {
    font-size: 1.4em;
    font-weight: bold;
  }
  .tagRank2 {
    font-size: 1.3em;
    font-weight: bold;
    color: #222;
  }
  .tagRank3 {
    font-size: 1.2em;
    font-weight: bold;
    color: #333;
  }
  .tagRank4 {
    font-size: 1.1em;
    color: #444;
  }
  .tagRank5 {
    font-size: 1em;
    color: #555;
  }
  .tagRank6 {
    font-size: 0.95em;
    color: #666;
  }
  .tagRank7 {
    font-size: 0.9em;
    color: #777;
  }
  .tagRank8 {
    font-size: 0.85em;
    color: #888;
  }
  .tagRank9 {
    font-size: 0.8em;
    color: #999;
  }
  .tagRank10 {
    font-size: 0.75em;
    color: #aaa;
  }
  
  a:hover {
    text-decoration: underline;
    position: relative;
    z-index: 50;
    color: #f00 !important;
  }
}
</style>