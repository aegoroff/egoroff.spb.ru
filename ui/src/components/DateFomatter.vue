<template>
  <span itemprop="datePublished" class="shortDate">{{ format() }}</span>
</template>

<script lang="ts">
import moment from 'moment'
import { Component, Prop, Vue } from 'vue-property-decorator'

@Component
export default class DateFormatter extends Vue {
  @Prop({
    type: String,
    required: true
  })
  public date!: string

  @Prop({
    type: String,
    required: true
  })
  public formatStr!: string

  format (): string {
    return moment(this.date).locale('ru').format(this.formatStr)
  }
}
</script>
