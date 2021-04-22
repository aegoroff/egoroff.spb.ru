<template>
  <highlight-code :lang="lang">{{ content }}</highlight-code>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'

@Component
export default class Highlighter extends Vue {
  @Prop() private lang!: string
  @Prop() private content!: string
}
</script>
