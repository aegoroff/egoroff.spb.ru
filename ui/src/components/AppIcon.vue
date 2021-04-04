<template>
  <font-awesome-icon v-bind:icon="icon"/>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'

@Component
export default class AppIcon extends Vue {
  @Prop({
    type: String,
    required: true
  })
  public icon!: string
}
</script>

<style scoped>

</style>
