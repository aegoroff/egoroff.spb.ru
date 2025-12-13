import { createApp, h } from 'vue'
import Vue2Filters from 'vue2-filters'
import VueHighlightJS from 'vue-highlight.js'
import VueSocialSharing from 'vue-social-sharing'
import App from './App.vue'
import AdminApp from './AdminApp.vue'
import { library } from '@fortawesome/fontawesome-svg-core'
import {
  faBook,
  faBriefcase,
  faSearch,
  faHome,
  faUser,
  faCalendarAlt,
  faDownload,
  faSignInAlt,
  faSignOutAlt,
  faTools,
  faTrashAlt
} from '@fortawesome/free-solid-svg-icons'
import { faGoogle, faGithub, faVk, faYandex } from '@fortawesome/free-brands-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import VueProgressBar from 'vue-progressbar'
import AppIcon from './components/AppIcon.vue'
import DateFormatter from '@/components/DateFomatter.vue'
import FromNow from '@/components/FromNow.vue'
import BlogNavigation from '@/components/BlogNavigation.vue'

import './App.scss'
import 'bootstrap/dist/css/bootstrap.min.css'
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import Search from '@/views/Search.vue'
import Profile from '@/views/Profile.vue'
import Social from '@/components/Social.vue'

// Highlight.js languages (All languages)
import 'vue-highlight.js/lib/allLanguages'

/*
* Import Highlight.js theme
* Find more: https://highlightjs.org/static/demo/
*/
import 'highlight.js/styles/googlecode.css'
import Highlighter from '@/components/Highlighter.vue'
import Alert from '@/components/Alert.vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import routes from '@/router'
import Downloads from '@/components/Downloads.vue'
import { createPinia } from 'pinia'
import mitt from 'mitt'

library.add(faBook, faBriefcase, faSearch, faHome, faUser, faCalendarAlt, faDownload, faSignInAlt, faSignOutAlt, faTools, faTrashAlt)
library.add(faGoogle, faGithub, faVk, faYandex)

// Создаем Pinia store
const pinia = createPinia()

// Создаем event bus
export const emitter = mitt()

const appElement = document.getElementById('app')

if (appElement) {
  const t = appElement.getAttribute('datafld')
  const vueApp = createApp({
    render() {
      return h(App, { title: t || '' })
    }
  })
  
  // Регистрируем глобальные компоненты
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.component('AppIcon', AppIcon)
  vueApp.component('DateFormatter', DateFormatter)
  vueApp.component('FromNow', FromNow)
  vueApp.component('BlogNavigation', BlogNavigation)
  vueApp.component('BlogAnnounces', BlogAnnounces)
  vueApp.component('BlogTitle', BlogTitle)
  vueApp.component('Highlighter', Highlighter)
  vueApp.component('Alert', Alert)
  vueApp.component('Social', Social)
  vueApp.component('Downloads', Downloads)
  vueApp.component('Search', Search)
  vueApp.component('Profile', Profile)
  
  // Используем плагины
  vueApp.use(pinia)
  vueApp.use(Vue2Filters)
  
  // VueProgressBar конфигурация
  const progressBarOptions = {
    color: '#bffaf3',
    failedColor: '#874b4b',
    thickness: '3px',
    transition: {
      speed: '0.2s',
      opacity: '0.6s',
      termination: 300
    },
    autoRevert: true,
    location: 'top',
    inverse: false
  }
  
  vueApp.use(VueProgressBar, progressBarOptions)
  vueApp.use(VueSocialSharing)
  vueApp.use(VueHighlightJS)
  
  // Добавляем event bus в глобальные свойства
  vueApp.config.globalProperties.emitter = emitter
  
  vueApp.mount('#app')
}

// Монтируем отдельные компоненты
const blogNavigationElement = document.getElementById('blogNavigation')
if (blogNavigationElement) {
  const vueApp = createApp(BlogNavigation)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#blogNavigation')
}

const portfolioDownloadsElement = document.getElementById('portfolioDownloads')
if (portfolioDownloadsElement) {
  const vueApp = createApp(Downloads)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#portfolioDownloads')
}

const socialElement = document.getElementById('social')
if (socialElement) {
  const title = socialElement.getAttribute('property')
  const vueApp = createApp({
    render() {
      return h(Social, {
        title: title || '',
        url: window.location.href,
        networks: ['vk']
      })
    }
  })
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#social')
}

const siteSearchElement = document.getElementById('siteSearch')
if (siteSearchElement) {
  const key = siteSearchElement.getAttribute('property')
  const cx = siteSearchElement.getAttribute('datafld')
  const urlParams = new URLSearchParams(window.location.search)
  const q = urlParams.get('q')
  
  const vueApp = createApp({
    render() {
      return h(Search, {
        key: key || '',
        cx: cx || '',
        query: q || ''
      })
    }
  })
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#siteSearch')
}

const userProfileElement = document.getElementById('userProfile')
if (userProfileElement) {
  const vueApp = createApp(Profile)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.component('AppIcon', AppIcon)
  vueApp.mount('#userProfile')
}

if (document.getElementById('blogcontainer') && window.location.hash) {
  const h = window.location.hash.substring(1)
  
  const vueApp = createApp({
    render() {
      return h(BlogAnnounces, { q: h })
    }
  })
  vueApp.component('DateFormatter', DateFormatter)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#blogcontainer')
  
  const blogTitleElement = document.getElementById('blogSmallTitle')
  if (blogTitleElement) {
    const e = h.split('=')
    const vueApp2 = createApp({
      render() {
        return h(BlogTitle, { text: 'все посты по метке: ' + e[1] })
      }
    })
    vueApp2.mount('#blogSmallTitle')
  }
}

const icons = document.querySelectorAll('i.icon[data-label]')
icons.forEach(x => {
  const label = x.getAttribute('data-label')
  const type = x.getAttribute('datatype')
  const icon = label === null ? '' : label.value
  const lib = type === null ? '' : type.value
  
  const vueApp = createApp({
    render() {
      return h(AppIcon, {
        icon: icon,
        lib: lib
      })
    }
  })
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount(x)
})

const dates = document.querySelectorAll('span.date[data-label]')
dates.forEach(x => {
  const label = x.getAttribute('data-label')
  const fmt = label === null ? 'LL' : label.value
  
  if (fmt === 'from-now') {
    const vueApp = createApp({
      render() {
        return h(FromNow, {
          date: x.textContent || ''
        })
      }
    })
    vueApp.mount(x)
  } else {
    const vueApp = createApp({
      render() {
        return h(DateFormatter, {
          date: x.textContent || '',
          formatStr: fmt
        })
      }
    })
    vueApp.mount(x)
  }
})

const langMap = new Map<string, string>()
langMap.set('asm', 'x86asm')
langMap.set('hq', 'cs')
langMap.set('parser', 'parser3')
langMap.set('php', 'parser3')

function replacementLang (lang: string): string {
  const l = langMap.get(lang)
  return l !== undefined ? l : lang
}

function mountHighlighting (prefix: string, x: Element): void {
  if (!x.className.startsWith(prefix)) {
    return
  }
  const lang = x.className.replace(prefix, '')
    .replace(';', '')
    
  const vueApp = createApp({
    render() {
      return h(Highlighter, {
        content: x.textContent || '',
        lang: replacementLang(lang)
      })
    }
  })
  vueApp.mount(x)
}

const snippets = document.querySelectorAll('pre, code')
snippets.forEach(x => {
  mountHighlighting('brush: ', x)
  mountHighlighting('language-', x)
})

const alerts = document.querySelectorAll('.alert')
alerts.forEach(x => {
  const type = x.getAttribute('data-label')
  const alert = type === null ? 'success' : type.value
  
  const vueApp = createApp({
    render() {
      return h(Alert, {
        content: x.textContent || '',
        type: alert
      })
    }
  })
  vueApp.mount(x)
})

const adminApp = document.getElementById('admin')
if (adminApp) {
  const router = createRouter({
    history: createWebHashHistory(),
    routes
  })
  
  const vueApp = createApp(AdminApp)
  vueApp.use(router)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#admin')
}