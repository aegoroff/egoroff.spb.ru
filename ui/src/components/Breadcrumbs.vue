<template>
  <div class="container" id="breadcrumbs" v-if="breadcrumbs.length > 0">
    <div class="row">
      <div class="col-lg-12">
        <b-breadcrumb>
          <b-breadcrumb-item v-for="section in breadcrumbs" :key="section.id"
                             v-bind:href="section.id === '/' ? section.id : '/'+ section.id + '/'">
            <font-awesome-icon v-bind:icon="section.icon"/>
            {{ section.title }}
          </b-breadcrumb-item>
          <b-breadcrumb-item active>{{ title }}</b-breadcrumb-item>
        </b-breadcrumb>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import { Section } from './Navigation.vue'

@Component
export default class Breadcrumbs extends Vue {
  @Prop() private breadcrumbs!: Array<Section>
  @Prop() private title!: string
}
</script>

<style scoped lang="scss">

</style>
