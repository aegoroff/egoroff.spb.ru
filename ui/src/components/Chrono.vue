<template>
  <div id="accordion" role="tablist">
    <b-card class="m-2" no-body v-for="y in years" :key="y.year">

      <b-card-header header-tag="header" class="p-1" role="tab">
        <b-button block v-b-toggle="'y' + y.year" variant="light">{{ y.year }}</b-button>
      </b-card-header>

      <b-collapse v-bind:id="'y' + y.year" accordion="blog-archive" role="tabpanel">
        <b-card-body>
          <b-list-group flush>
            <b-list-group-item v-bind:href="'#year=' + y.year"
                               v-on:click="updateYear(y.year, 1)"
                               class="d-flex justify-content-between align-items-center">
              За весь год
              <b-badge variant="primary" pill>{{ y.posts }}</b-badge>
            </b-list-group-item>

            <b-list-group-item
              v-for="m in y.months"
              :key="m.month"
              v-on:click="updateYearMonth(y.year, m.month, 1)"
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
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import moment from 'moment'
import { bus } from '@/main'

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

  created (): void {
    bus.$on('pageChanged', (data: number) => {
      const parts = window.location.hash.substr(1).split('&')

      let y = 0
      let m = 0
      for (const part of parts) {
        const elts = part.split('=')
        if (elts[0] === 'year') {
          y = parseInt(elts[1], 10)
        }
        if (elts[0] === 'month') {
          m = parseInt(elts[1], 10)
        }
      }

      if (isNaN(y) || isNaN(m)) {
        return
      }

      if (m === 0 && y > 0) {
        this.updateYear(y, data)
      } else if (m > 0 && y > 0) {
        this.updateYearMonth(y, m, data)
      }
    })
  }

  updateYear (year: number, page: number): void {
    bus.$emit('dateSelectionChanged')
    const ba = new BlogAnnounces({
      propsData: {
        q: `year=${year}&page=${page}`
      }
    })
    ba.$mount('#blogcontainer')

    const bt = new BlogTitle({
      propsData: {
        text: `записи за ${year} год`
      }
    })
    bt.$mount('#blogSmallTitle')
  }

  updateYearMonth (year: number, month: number, page: number): void {
    bus.$emit('dateSelectionChanged')
    const ba = new BlogAnnounces({
      propsData: {
        q: `year=${year}&month=${month}&page=${page}`
      }
    })
    ba.$mount('#blogcontainer')

    const mnt = moment(new Date(year, month, 1)).locale('ru')
    const bt = new BlogTitle({
      propsData: {
        text: `записи за ${mnt.format('MMMM YYYY')}`
      }
    })
    bt.$mount('#blogSmallTitle')
  }
}

</script>
