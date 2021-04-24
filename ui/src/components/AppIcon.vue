<template>
  <i>
    <font-awesome-icon v-if="!lib" v-bind:icon="icon"/>
    <font-awesome-icon v-if="lib" v-bind:icon="[lib, icon]"/>
  </i>
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

  @Prop({
    type: String,
    required: false
  })
  public lib!: string
}
</script>
