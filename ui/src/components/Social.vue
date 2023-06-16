<template>
  <div>
    <ShareNetwork class="btn btn-sm" v-for="net in networks" :key="net" v-bind:network="net" v-bind:url="url"
                  v-bind:title="title">
      <font-awesome-icon v-bind:icon="['fab', net]"></font-awesome-icon>
    </ShareNetwork>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'

@Component
export default class Social extends Vue {
  @Prop() private url!: string
  @Prop() private title!: string
  @Prop() private networks!: Array<string>

  mounted (): void {
    this.networks = new Array<string>()
    this.networks.push('vk')
  }
}
</script>
