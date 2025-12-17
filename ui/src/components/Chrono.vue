<template>
  <div class="accordion" id="accordion">
    <div class="card m-2" v-for="y in years" :key="y.year">
      <!-- Header -->
      <div class="card-header p-1">
        <button
          class="btn btn-light w-100"
          type="button"
          data-bs-toggle="collapse"
          :data-bs-target="'#y' + y.year"
        >
          {{ y.year }}
        </button>
      </div>

      <!-- Collapse -->
      <div :id="'y' + y.year" class="collapse" data-bs-parent="#accordion">
        <div class="card-body">
          <div class="list-group list-group-flush">
            <!-- Whole year -->
            <a
              class="list-group-item d-flex justify-content-between align-items-center"
              :href="'#year=' + y.year"
              @click.prevent="updateYear(y.year, 1)"
            >
              За весь год
              <span class="badge bg-primary rounded-pill">
                {{ y.posts }}
              </span>
            </a>

            <!-- Months -->
            <a
              v-for="m in y.months"
              :key="m.month"
              class="list-group-item d-flex justify-content-between align-items-center"
              :href="'#year=' + y.year + '&month=' + m.month"
              @click.prevent="updateYearMonth(y.year, m.month, 1)"
            >
              {{ monthName(m.month) }}
              <span class="badge bg-secondary rounded-pill">
                {{ m.posts }}
              </span>
            </a>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { createApp } from 'vue'
import { emitter } from '@/main'
import dayjs from 'dayjs';
import 'dayjs/locale/ru';
dayjs.locale('ru');

import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import { Year } from '@/models/blog'

const props = defineProps<{
  years: Year[]
}>()

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

const updateYear = (year: number, page: number) => {
  const params = new URLSearchParams(window.location.hash.slice(1))
  params.set('year', String(year))
  if (page > 1) {
    params.set('page', String(page))
  } else {
    params.delete('page')
  }
  params.delete('tag')

  window.location.hash = params.toString()

  emitter.emit('dateSelectionChanged')

  createApp(BlogAnnounces, {
    q: `year=${year}&page=${page}`
  }).mount('#blogcontainer')

  createApp(BlogTitle, {
    text: `записи за ${year} год`
  }).mount('#blogSmallTitle')
}

const updateYearMonth = (year: number, month: number, page: number) => {
  const params = new URLSearchParams(window.location.hash.slice(1))
  params.set('year', String(year))
  params.set('month', String(month))
  if (page > 1) {
    params.set('page', String(page))
  } else {
    params.delete('page')
  }
  params.delete('tag')

  emitter.emit('dateSelectionChanged')

  createApp(BlogAnnounces, {
    q: `year=${year}&month=${month}&page=${page}`
  }).mount('#blogcontainer')

  const m = dayjs(new Date(year, month - 1, 1))

  createApp(BlogTitle, {
    text: `записи за ${m.format('MMMM YYYY')}`
  }).mount('#blogSmallTitle')
}

onMounted(() => {
  emitter.on('pageChanged', (page: number) => {
    const parts = window.location.hash.substring(1).split('&')

    let y = 0
    let m = 0

    for (const part of parts) {
      const [key, value] = part.split('=')
      if (key === 'year') y = Number(value)
      if (key === 'month') m = Number(value)
    }

    if (!y) return

    if (!m) {
      updateYear(y, page)
    } else {
      updateYearMonth(y, m, page)
    }
  })
})
</script>