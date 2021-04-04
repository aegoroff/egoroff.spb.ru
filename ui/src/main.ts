import Vue from 'vue'
import { BootstrapVue, IconsPlugin } from 'bootstrap-vue/src'
import App from './App.vue'
import { library } from '@fortawesome/fontawesome-svg-core'
import {
  faBook,
  faBriefcase,
  faSearch,
  faHome,
  faUser,
  faCalendarAlt
} from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import VueProgressBar from 'vue-progressbar'
import AppIcon from './components/AppIcon.vue'
import DateFormatter from '@/components/DateFomatter.vue'

import './App.scss'

Vue.config.productionTip = false
Vue.use(BootstrapVue)
Vue.use(IconsPlugin)
Vue.use(VueProgressBar)

library.add(faBook, faBriefcase, faSearch, faHome, faUser, faCalendarAlt)
Vue.component('font-awesome-icon', FontAwesomeIcon)

const app = new App()
app.$mount('#app')

const calendars = document.querySelectorAll('.calendar')
calendars.forEach(x => {
  const calendar = new AppIcon({
    propsData: {
      icon: 'calendar-alt'
    }
  })
  calendar.$mount(x)
})

const shortDate = document.querySelectorAll('.shortDate')
shortDate.forEach(x => {
  new DateFormatter({
    propsData: {
      date: x.innerHTML,
      formatStr: 'LL'
    }
  }).$mount(x)
})

const longDate = document.querySelectorAll('.longDate')
longDate.forEach(x => {
  new DateFormatter({
    propsData: {
      date: x.innerHTML,
      formatStr: 'LLL'
    }
  }).$mount(x)
})
