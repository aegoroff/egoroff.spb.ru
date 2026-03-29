declare module 'vue-progressbar' {
  import { Plugin } from 'vue'

  interface ProgressMethods {
    start(): void;
    finish(): void;
    fail(): void;
  }

  interface VueProgressBar {
    install: Plugin['install'];
  }

  const VueProgressBar: VueProgressBar;
  export default VueProgressBar;

  declare module '@vue/runtime-core' {
    interface ComponentCustomProperties {
      $Progress: ProgressMethods;
    }
  }
}
