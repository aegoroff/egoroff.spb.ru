declare module 'vue-progressbar' {
  import { Plugin } from 'vue'

  interface ProgressMethods {
    start(): void;
    finish(): void;
    fail(): void;
  }

  interface ProgressOptions {
    color: string;
    failedColor: string;
    thickness: string;
    transition: {
      speed: string;
      opacity: string;
      termination: number;
    };
    autoRevert: boolean;
    location: string;
    inverse: boolean;
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