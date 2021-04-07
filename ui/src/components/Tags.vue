<template>
  <div class="tags">
    <ul class="list-inline">
      <li class="list-inline-item" v-for="tag in tags" :key="tag.title">
        <a
          v-bind:href="'/blog/#tag=' + tag.title"
          v-bind:class="tag.level"
          v-bind:id="'t_' + tag.title">{{ tag.title }}</a>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'

export class Tag {
  public title!: string
  public level!: string
}

@Component
export default class Tags extends Vue {
  @Prop() private tags!: Array<Tag>
}

</script>
