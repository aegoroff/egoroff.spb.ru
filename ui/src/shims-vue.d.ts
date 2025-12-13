declare module '*.vue' {
  import { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module 'vue3-highlightjs' {
  import { Plugin } from 'vue'
  export const Vue3Highlightjs: Plugin
}