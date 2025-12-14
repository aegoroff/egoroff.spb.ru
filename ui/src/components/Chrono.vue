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
import { defineComponent, PropType } from 'vue'
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
  setup() {
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
      return months[month] || ''
    }

    const updateYear = (year: number, page: number): void => {
      emitter.emit('dateSelectionChanged')
      
      const params = new URLSearchParams(window.location.hash.substring(1));
      params.set('year', year.toString());
      params.set('page', page.toString());
      
      window.location.hash = '#' + params.toString();
      updateContent(year, undefined, page)
    }

    const updateYearMonth = (year: number, month: number, page: number): void => {
      emitter.emit('dateSelectionChanged')
      
      const params = new URLSearchParams(window.location.hash.substring(1));
      params.set('year', year.toString());
      params.set('month', month.toString());
      params.set('page', page.toString());
      
      window.location.hash = '#' + params.toString();
      updateContent(year, month, page)
    }
    
    const updateContent = (year: number, month?: number, page: number = 1): void => {
      const blogContainer = document.getElementById('blogcontainer')
      const blogTitle = document.getElementById('blogSmallTitle')
      
      let q = `year=${year}&page=${page}`
      let titleText = `все посты за ${year} год`
      
      if (month) {
        q = `year=${year}&month=${month}&page=${page}`
        const monthNames = ['Январь', 'Февраль', 'Март', 'Апрель', 'Май', 'Июнь', 'Июль', 'Август', 'Сентябрь', 'Октябрь', 'Ноябрь', 'Декабрь']
        titleText = `все посты за ${monthNames[month - 1]} ${year} года`
      }
      
      if (blogContainer) {
        blogContainer.innerHTML = `<blog-announces q="${q}"></blog-announces>`
      }
      if (blogTitle) {
        blogTitle.innerHTML = `<blog-title text="${titleText}"></blog-title>`
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