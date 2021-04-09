<template>
  <small id="blogSmallTitle">{{ text }}</small>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'

@Component
export default class BlogTitle extends Vue {
  @Prop() private text!: string
}
</script>
