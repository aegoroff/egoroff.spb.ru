<template>
  <div class="container" id="breadcrumbs" v-if="breadcrumbs.length > 0">
    <div class="row">
      <div class="col-lg-12">
        <nav aria-label="breadcrumb">
          <ol class="breadcrumb">
            <li class="breadcrumb-item" v-for="section in breadcrumbs" :key="section.id">
              <a :href="section.id === '/' ? section.id : '/'+ section.id + '/'">
                <font-awesome-icon :icon="section.icon"/>
                {{ section.title }}
              </a>
            </li>
            <li class="breadcrumb-item active" aria-current="page">{{ title }}</li>
          </ol>
        </nav>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Section } from '@/models/common';


defineProps<{
  breadcrumbs: Section[]
  title: string
}>()
</script>

<style scoped lang="scss">

</style>