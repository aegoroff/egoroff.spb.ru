import { createApp, h } from 'vue'
import Vue3Filters from 'vue3-filters'
import { Vue3Highlightjs } from 'vue3-highlightjs'
import VueSocialSharing from 'vue3-social-sharing'
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
import {Vue3ProgressPlugin} from '@marcoschulte/vue3-progress';
import AppIcon from './components/AppIcon.vue'
import DateFormatter from '@/components/DateFomatter.vue'
import FromNow from '@/components/FromNow.vue'
import BlogNavigation from '@/components/BlogNavigation.vue'
import './App.scss'
import 'bootstrap/dist/css/bootstrap.min.css'
import 'bootstrap/dist/js/bootstrap.bundle.min.js'
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import Search from '@/views/Search.vue'
import Profile from '@/views/Profile.vue'
import Social from '@/components/Social.vue'
import 'highlight.js/styles/github.css'
import Highlighter from '@/components/Highlighter.vue'
import Alert from '@/components/Alert.vue'
import { createAdminRouter } from '@/router'
import Downloads from '@/components/Downloads.vue'
import { createPinia } from 'pinia'
import mitt from 'mitt'
import hljs from 'highlight.js/lib/core'
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import xml from 'highlight.js/lib/languages/xml'
import css from 'highlight.js/lib/languages/css'
import scss from 'highlight.js/lib/languages/scss'
import json from 'highlight.js/lib/languages/json'
import bash from 'highlight.js/lib/languages/bash'
import python from 'highlight.js/lib/languages/python'
import java from 'highlight.js/lib/languages/java'
import csharp from 'highlight.js/lib/languages/csharp'
import php from 'highlight.js/lib/languages/php'
import sql from 'highlight.js/lib/languages/sql'
import go from 'highlight.js/lib/languages/go'

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('css', css)
hljs.registerLanguage('scss', scss)
hljs.registerLanguage('json', json)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('python', python)
hljs.registerLanguage('java', java)
hljs.registerLanguage('csharp', csharp)
hljs.registerLanguage('php', php)
hljs.registerLanguage('sql', sql)
hljs.registerLanguage('go', go)

library.add(faBook, faBriefcase, faSearch, faHome, faUser, faCalendarAlt, faDownload, faSignInAlt, faSignOutAlt, faTools, faTrashAlt)
library.add(faGoogle, faGithub, faVk, faYandex)
const pinia = createPinia()
export const emitter = mitt()

const appElement = document.getElementById('app')
if (appElement) {
  const t = appElement.getAttribute('datafld')
  const vueApp = createApp({
    render() {
      return h(App, { title: t || '' })
    }
  })
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
  vueApp.use(pinia)
  vueApp.use(Vue3Filters, {})
  vueApp.use(Vue3Highlightjs)
  
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
  
  vueApp.use(Vue3ProgressPlugin, progressBarOptions)
  vueApp.use(VueSocialSharing)
  vueApp.config.globalProperties.emitter = emitter
  
  vueApp.mount('#app')
}

if (document.getElementById('blogNavigation')) {
  const vueApp = createApp(BlogNavigation)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#blogNavigation')
}

if (document.getElementById('blogcontainer') && window.location.hash) {
  const hash = window.location.hash.substring(1)
  const params = new URLSearchParams(hash)
  const tag = params.get('tag')
  const year = params.get('year')
  const month = params.get('month')
  const page = params.get('page') || '1'
  
  let q = ''
  if (tag) {
    q = `tag=${encodeURIComponent(tag)}&page=${page}`
  } else if (year) {
    if (month) {
      q = `year=${encodeURIComponent(year)}&month=${encodeURIComponent(month)}&page=${page}`
    } else {
      q = `year=${encodeURIComponent(year)}&page=${page}`
    }
  } else if (page !== '1') {
    q = `page=${page}`
  }
  
  if (q) {
    const vueApp = createApp({
      render() {
        return h(BlogAnnounces, { q: q })
      }
    })
    vueApp.component('DateFormatter', DateFormatter)
    vueApp.component('font-awesome-icon', FontAwesomeIcon)
    vueApp.mount('#blogcontainer')
    
    const blogTitleElement = document.getElementById('blogSmallTitle')
    if (blogTitleElement) {
      let titleText = 'тут я пишу'
      if (tag) {
        titleText = `все посты по метке: ${tag}`
      } else if (year) {
        if (month) {
          const monthNames = ['Январь', 'Февраль', 'Март', 'Апрель', 'Май', 'Июнь', 'Июль', 'Август', 'Сентябрь', 'Октябрь', 'Ноябрь', 'Декабрь']
          titleText = `все посты за ${monthNames[parseInt(month) - 1]} ${year} года`
        } else {
          titleText = `все посты за ${year} год`
        }
      }
      
      const vueApp2 = createApp({
        render() {
          return h(BlogTitle, { text: titleText })
        }
      })
      vueApp2.mount('#blogSmallTitle')
    }
  }
}

if (document.getElementById('portfolioDownloads')) {
  const vueApp = createApp(Downloads)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#portfolioDownloads')
}

if (document.getElementById('social')) {
  const title = document.getElementById('social')?.getAttribute('property')
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

if (document.getElementById('siteSearch')) {
  const key = document.getElementById('siteSearch')?.getAttribute('property')
  const cx = document.getElementById('siteSearch')?.getAttribute('datafld')
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

if (document.getElementById('userProfile')) {
  const vueApp = createApp(Profile)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.component('AppIcon', AppIcon)
  vueApp.mount('#userProfile')
}

const icons = document.querySelectorAll('i.icon[data-label]')
icons.forEach(x => {
  const label = x.getAttribute('data-label')
  const type = x.getAttribute('datatype')
  const icon = label || ''
  const lib = type || ''
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
  const fmt = label === null ? 'LL' : label
  const text = x.textContent?.trim() ?? ''
  if (fmt === 'from-now') {
    const vueApp = createApp({
      render() {
        return h(FromNow, {
          date: text
        })
      }
    })
    vueApp.mount(x)
  } else {
    const vueApp = createApp({
      render() {
        return h(DateFormatter, {
          date: text,
          formatStr: fmt
        })
      }
    })
    vueApp.mount(x)
  }
})

function mountHighlighting (prefix: string, x: Element): void {
  if (!x.className.startsWith(prefix)) {
    return
  }
  const lang = x.className.replace(prefix, '')
    .replace(';', '')
  const text = x.textContent
  const vueApp = createApp({
    render() {
      return h(Highlighter, {
        content: text || '',
        lang: lang
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
  const alert = type || 'success'
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

if (document.getElementById('admin')) {
  const router = createAdminRouter()
  const vueApp = createApp(AdminApp)
  vueApp.use(router)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#admin')
}