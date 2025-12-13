import { createApp, h } from 'vue'
import Vue3Filters from 'vue3-filters'
import { Vue3Highlightjs } from 'vue3-highlightjs'
import 'highlight.js/styles/googlecode.css'
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

/*
* Import Highlight.js theme
* Find more: https://highlightjs.org/static/demo/
*/
import 'highlight.js/styles/googlecode.css'
import Highlighter from '@/components/Highlighter.vue'
import Alert from '@/components/Alert.vue'
import router from '@/router'
import Downloads from '@/components/Downloads.vue'
import { createPinia } from 'pinia'
import mitt from 'mitt'

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
  vueApp.use(Vue3Highlightjs)

  vueApp.config.globalProperties.emitter = emitter
  vueApp.mount('#app')
}

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
  const hash = window.location.hash.substring(1)

  const vueApp = createApp({
    render() {
      return h(BlogAnnounces, { q: hash })
    }
  })
  vueApp.component('DateFormatter', DateFormatter)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#blogcontainer')

  const blogTitleElement = document.getElementById('blogSmallTitle')
  if (blogTitleElement) {
    const e = hash.split('=')
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
  const fmt = label || 'LL'

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

const adminApp = document.getElementById('admin')
if (adminApp) {
  const vueApp = createApp(AdminApp)
  vueApp.use(router)
  vueApp.component('font-awesome-icon', FontAwesomeIcon)
  vueApp.mount('#admin')
}
