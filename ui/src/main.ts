import Vue from 'vue'
import VueTypeScriptInject from 'vue-typescript-inject'
import VueGtag from 'vue-gtag'
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
import FromNow from '@/components/FromNow.vue'

import './App.scss'

Vue.config.productionTip = false
Vue.use(VueTypeScriptInject)
Vue.use(BootstrapVue)
Vue.use(IconsPlugin)
Vue.use(VueProgressBar)
Vue.use(VueGtag, {
  config: { id: 'UA-145548-1' }
})

library.add(faBook, faBriefcase, faSearch, faHome, faUser, faCalendarAlt)
Vue.component('font-awesome-icon', FontAwesomeIcon)

const app = new App()
app.$mount('#app')

const icons = document.querySelectorAll('i.icon[data-label]')
icons.forEach(x => {
  const label = x.attributes.getNamedItem('data-label')
  const icon = label === null ? '' : label.value
  const appIcon = new AppIcon({
    propsData: {
      icon: icon
    }
  })
  appIcon.$mount(x)
})

const shortDate = document.querySelectorAll('span.date[data-label]')
shortDate.forEach(x => {
  const label = x.attributes.getNamedItem('data-label')
  const fmt = label === null ? 'LL' : label.value
  new DateFormatter({
    propsData: {
      date: x.innerHTML,
      formatStr: fmt
    }
  }).$mount(x)
})

const fromNow = document.querySelectorAll('.date-from-now')
fromNow.forEach(x => {
  new FromNow({
    propsData: {
      date: x.innerHTML
    }
  }).$mount(x)
})
