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
          :aria-expanded="y.year === currentYear"
          :aria-controls="'y' + y.year"
        >
          {{ y.year }}
        </button>
      </div>

      <!-- Collapse -->
      <div :id="'y' + y.year" class="collapse" :class="{ show: y.year === currentYear }" data-bs-parent="#accordion">
        <div class="card-body">
          <div class="list-group list-group-flush">
            <!-- Whole year -->
            <a
              class="list-group-item d-flex justify-content-between align-items-center"
              :href="`/blog/?year=${y.year}`"
              data-blog-link
              :class="{ active: y.year === currentYear && !currentMonth }"
              :aria-current="y.year === currentYear && !currentMonth ? 'true' : undefined"
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
              :href="`/blog/?year=${y.year}&month=${m.month}`"
              data-blog-link
              :class="{ active: y.year === currentYear && m.month === currentMonth }"
              :aria-current="y.year === currentYear && m.month === currentMonth ? 'true' : undefined"
            >
              {{ formatMonthName(m.month) }}
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
import { formatMonthName } from '@/util'
import type { Year } from '@/models/blog'

defineProps<{
  years: Year[]
  currentYear: number
  currentMonth: number
}>()
</script>
