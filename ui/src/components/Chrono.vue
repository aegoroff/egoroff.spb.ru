<template>
  <div id="accordion" role="tablist">
    <b-card class="m-2" no-body v-for="y in years" :key="y.year">

      <b-card-header header-tag="header" class="p-1" role="tab">
        <b-button block v-b-toggle="'y' + y.year" variant="light">{{y.year}}</b-button>
      </b-card-header>

      <b-collapse v-bind:id="'y' + y.year" accordion="blog-archive" role="tabpanel">
        <b-card-body>
          <b-list-group flush>
            <b-list-group-item v-bind:href="'#year=' + y.year" class="d-flex justify-content-between align-items-center">
              За весь год
              <b-badge variant="primary" pill>{{ y.posts }}</b-badge>
            </b-list-group-item>

            <b-list-group-item
              v-for="m in y.months"
              :key="m.month"
              v-bind:href="'#year=' +y.year + '&month=' + m.month"
              class="d-flex justify-content-between align-items-center">
              {{ m.name }}
              <b-badge variant="secondary" pill>{{ m.posts }}</b-badge>
            </b-list-group-item>
          </b-list-group>
        </b-card-body>
      </b-collapse>
    </b-card>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'

export class Month {
  public month!: number
  public name!: string
  public posts!: number
}

export class Year {
  public year!: number
  public posts!: number
  public months!: Array<Month>
}

@Component
export default class Chrono extends Vue {
  @Prop() private years!: Array<Year>
}

</script>
