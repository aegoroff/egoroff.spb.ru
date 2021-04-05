<template>
  <span class="date-from-now">{{ format() }}</span>
</template>

<script lang="ts">
import moment from 'moment'
import { Component, Prop, Vue } from 'vue-property-decorator'

@Component
export default class FromNow extends Vue {
  @Prop({
    type: String,
    required: true
  })
  public date!: string

  format (): string {
    return moment(this.date).locale('ru').fromNow()
  }
}
</script>
