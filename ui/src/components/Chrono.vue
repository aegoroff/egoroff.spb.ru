<template>
  <div class="accordion" id="blogArchiveAccordion">
    <div class="accordion-item" v-for="y in years" :key="y.year">
      <h2 class="accordion-header">
        <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" 
                :data-bs-target="'#collapse' + y.year">
          {{ y.year }}
        </button>
      </h2>
      <div :id="'collapse' + y.year" class="accordion-collapse collapse" 
           data-bs-parent="#blogArchiveAccordion">
        <div class="accordion-body">
          <ul class="list-group list-group-flush">
            <li class="list-group-item d-flex justify-content-between align-items-center">
              <a :href="'#year=' + y.year" @click.prevent="updateYear(y.year, 1)">
                За весь год
              </a>
              <span class="badge bg-primary rounded-pill">{{ y.posts }}</span>
            </li>
            <li class="list-group-item d-flex justify-content-between align-items-center"
                v-for="m in y.months" :key="m.month">
              <a :href="'#year=' + y.year + '&month=' + m.month" 
                 @click.prevent="updateYearMonth(y.year, m.month, 1)">
                {{ monthName(m.month) }}
              </a>
              <span class="badge bg-secondary rounded-pill">{{ m.posts }}</span>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType, onMounted } from 'vue'
import moment from 'moment'
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import { emitter } from '@/main'

export class Month {
  public month!: number
  public posts!: number
}

export class Year {
  public year!: number
  public posts!: number
  public months!: Array<Month>
}

export default defineComponent({
  name: 'Chrono',
  props: {
    years: {
      type: Array as PropType<Year[]>,
      required: true
    }
  },
  setup(props) {
    const months: Record<number, string> = {
      1: 'Январь',
      2: 'Февраль',
      3: 'Март',
      4: 'Апрель',
      5: 'Май',
      6: 'Июнь',
      7: 'Июль',
      8: 'Август',
      9: 'Сентябрь',
      10: 'Октябрь',
      11: 'Ноябрь',
      12: 'Декабрь'
    }

    const monthName = (month: number): string => {
      return months[month]
    }

    onMounted(() => {
      emitter.on('pageChanged', (data: any) => {
        const page = typeof data === 'number' ? data : 1
        const parts = window.location.hash.substring(1).split('&')

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
          updateYear(y, page)
        } else if (m > 0 && y > 0) {
          updateYearMonth(y, m, page)
        }
      })
    })

    const updateYear = (year: number, page: number): void => {
      emitter.emit('dateSelectionChanged')
      const blogContainer = document.getElementById('blogcontainer')
      const blogTitle = document.getElementById('blogSmallTitle')
      
      if (blogContainer) {
        // В реальном приложении здесь будет создание компонента
        // через createApp и монтирование
        blogContainer.innerHTML = `<blog-announces q="year=${year}&page=${page}"></blog-announces>`
      }
      
      if (blogTitle) {
        blogTitle.innerHTML = `<blog-title text="записи за ${year} год"></blog-title>`
      }
    }

    const updateYearMonth = (year: number, month: number, page: number): void => {
      emitter.emit('dateSelectionChanged')
      const blogContainer = document.getElementById('blogcontainer')
      const blogTitle = document.getElementById('blogSmallTitle')
      
      if (blogContainer) {
        blogContainer.innerHTML = `<blog-announces q="year=${year}&month=${month}&page=${page}"></blog-announces>`
      }
      
      if (blogTitle) {
        const m = month - 1
        const mnt = moment(new Date(year, m, 1)).locale('ru')
        blogTitle.innerHTML = `<blog-title text="записи за ${mnt.format('MMMM YYYY')}"></blog-title>`
      }
    }

    return {
      monthName,
      updateYear,
      updateYearMonth
    }
  }
})
</script>