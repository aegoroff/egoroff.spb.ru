declare module 'vue-progressbar' {
  import { Plugin } from 'vue'

  export const install: Plugin

  interface ProgressMethods {
    start(): void;
    finish(): void;
    fail(): void;
  }

  declare module '@vue/runtime-core' {
    interface ComponentCustomProperties {
      $Progress: ProgressMethods;
    }
  }
}