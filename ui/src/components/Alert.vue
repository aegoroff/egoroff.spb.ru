<template>
  <div>
    <b-alert show dismissible v-bind:variant="type">{{ content }}</b-alert>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'

@Component
export default class Alert extends Vue {
  @Prop() private type!: string
  @Prop() private content!: string
}
</script>
