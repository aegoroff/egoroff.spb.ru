import { createApp } from "vue";
import VueSocialSharing from "vue3-social-sharing";
import App from "./App.vue";
import AdminApp from "./AdminApp.vue";
import { library } from "@fortawesome/fontawesome-svg-core";
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
  faTrashAlt,
} from "@fortawesome/free-solid-svg-icons";
import {
  faGoogle,
  faGithub,
  faVk,
  faYandex,
} from "@fortawesome/free-brands-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
import { Vue3ProgressPlugin } from "@marcoschulte/vue3-progress";
import AppIcon from "./components/AppIcon.vue";
import DateFormatter from "@/components/DateFormatter.vue";
import BlogNavigation from "@/components/BlogNavigation.vue";
import "./App.scss";
import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap/dist/js/bootstrap.bundle.min.js";
import BlogAnnounces from "@/components/BlogAnnounces.vue";
import BlogTitle from "@/components/BlogTitle.vue";
import Search from "@/views/Search.vue";
import Profile from "@/views/Profile.vue";
import Social from "@/components/Social.vue";
import Alert from "@/components/Alert.vue";
import { createAdminRouter } from "@/router";
import Downloads from "@/components/Downloads.vue";
import mitt from "mitt";

import "highlight.js/lib/common";
import "highlight.js/styles/github.css";
import Highlighter from "@/components/Highlighter.vue";

library.add(
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
);
library.add(faGoogle, faGithub, faVk, faYandex);

export type Events = {
  pageChanged: number;
  dateSelectionChanged: void;
  postDeleted: void;
  downloadCreated: void;
  downloadDeleted: void;
};
export const emitter = mitt<Events>();

const appElement = document.getElementById("app");
if (appElement) {
  const t = appElement.getAttribute("datafld");
  const vueApp = createApp(App, { title: t || "" });
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.component("AppIcon", AppIcon);
  vueApp.component("DateFormatter", DateFormatter);
  vueApp.component("BlogNavigation", BlogNavigation);
  vueApp.component("BlogAnnounces", BlogAnnounces);
  vueApp.component("BlogTitle", BlogTitle);
  vueApp.component("Highlighter", Highlighter);
  vueApp.component("Alert", Alert);
  vueApp.component("Social", Social);
  vueApp.component("Downloads", Downloads);
  vueApp.component("Search", Search);
  vueApp.component("Profile", Profile);

  vueApp.use(Vue3ProgressPlugin);
  vueApp.use(VueSocialSharing as any);
  vueApp.config.globalProperties.emitter = emitter;

  vueApp.mount("#app");
}

if (document.getElementById("blogNavigation")) {
  const vueApp = createApp(BlogNavigation);
  vueApp.mount("#blogNavigation");
}

if (document.getElementById("blogcontainer") && window.location.hash) {
  const hash = window.location.hash.substring(1);
  const vueApp = createApp(BlogAnnounces, { q: hash });
  vueApp.component("DateFormatter", DateFormatter);
  vueApp.mount("#blogcontainer");

  if (document.getElementById("blogSmallTitle")) {
    const e = hash.split("=");
    let titleText = `все посты по метке: ${e[1]}`;

    const vueApp2 = createApp(BlogTitle, { text: titleText });
    vueApp2.mount("#blogSmallTitle");
  }
}

if (document.getElementById("portfolioDownloads")) {
  const vueApp = createApp(Downloads);
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#portfolioDownloads");
}

const social = document.getElementById("social");
if (social) {
  const title = social.getAttribute("property");
  const vueApp = createApp(Social, {
    title: title || "",
    url: window.location.href,
    networks: ["vk"],
  });
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#social");
}

const search = document.getElementById("siteSearch");
if (search) {
  const apiKey = search.getAttribute("property");
  const cx = search.getAttribute("datafld");
  const urlParams = new URLSearchParams(window.location.search);
  const q = urlParams.get("q");
  const vueApp = createApp(Search, {
    apiKey: apiKey || "",
    cx: cx || "",
    query: q || "",
  });
  vueApp.mount("#siteSearch");
}

if (document.getElementById("userProfile")) {
  const vueApp = createApp(Profile);
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.component("AppIcon", AppIcon);
  vueApp.mount("#userProfile");
}

document.querySelectorAll("i.icon[data-label]").forEach((x) => {
  const label = x.getAttribute("data-label");
  const type = x.getAttribute("datatype");
  const icon = label || "";
  const lib = type || "";
  const vueApp = createApp(AppIcon, { icon: icon, lib: lib });
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount(x);
});

document.querySelectorAll("span.date[data-label]").forEach((x) => {
  const label = x.getAttribute("data-label");
  const fmt = label === null ? "LL" : label;
  const text = x.textContent?.trim() ?? "";
  const vueApp = createApp(DateFormatter, { date: text, formatStr: fmt });
  vueApp.mount(x);
});

function mountHighlighting(prefix: string, el: Element): void {
  if (!el.className.startsWith(prefix)) return;

  const lang = el.className.replace(prefix, "").replace(";", "").trim();
  const app = createApp(Highlighter, { content: el.textContent ?? "", lang: lang, });
  el.textContent = "";
  app.mount(el);
}

document.querySelectorAll("pre, code").forEach((el) => {
  mountHighlighting("brush: ", el);
  mountHighlighting("language-", el);
});

document.querySelectorAll(".alert").forEach((x) => {
  const type = x.getAttribute("data-label");
  const alert = type || "success";
  const vueApp = createApp(Alert, { content: x.textContent || "", type: alert });
  vueApp.mount(x);
});

if (document.getElementById("admin")) {
  const router = createAdminRouter();
  const vueApp = createApp(AdminApp);
  vueApp.use(router);
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#admin");
}
