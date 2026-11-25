import "bootstrap/dist/css/bootstrap.min.css";
import "@marcoschulte/vue3-progress/style";
import "./styles/site.scss";
import "highlight.js/styles/github.css";
import { createApp, h, ref } from "vue";
import type { App as VueApp, Component } from "vue";
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
  faUsers,
  faArrowRight,
  faPen,
  faExternalLinkAlt,
} from "@fortawesome/free-solid-svg-icons";
import {
  faGoogle,
  faGithub,
  faVk,
  faTelegram,
  faYandex,
} from "@fortawesome/free-brands-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
import { Vue3ProgressPlugin } from "@marcoschulte/vue3-progress";
import AppIcon from "./components/AppIcon.vue";
import DateFormatter from "@/components/DateFormatter.vue";
import BlogNavigation from "@/components/BlogNavigation.vue";
import { enableBlogNavigation } from "@/blogNavigation";
// Import Bootstrap JS bundle (includes modal, dropdown, collapse, tab)
import "bootstrap/dist/js/bootstrap.bundle.min.js";
import SearchView from "@/views/SearchView.vue";
import ProfileView from "@/views/ProfileView.vue";
import SocialButtons from "@/components/SocialButtons.vue";
import { createAdminRouter } from "@/router";
import DownloadsList from "@/components/DownloadsList.vue";

import "highlight.js/lib/common";
import CodeHighlighter from "@/components/CodeHighlighter.vue";

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
  faTrashAlt,
  faUsers,
  faArrowRight,
  faPen,
  faExternalLinkAlt
);
library.add(faGoogle, faGithub, faVk, faTelegram, faYandex);

const appElement = document.getElementById("app");
const appTitle = ref(appElement?.getAttribute("datafld") ?? "");
if (appElement) {
  const vueApp = createApp(() => h(App, { title: appTitle.value }));
  vueApp.component("font-awesome-icon", FontAwesomeIcon as Component);
  vueApp.use(Vue3ProgressPlugin);
  vueApp.mount(appElement);
}

const portfolioDownloads = document.getElementById("portfolioDownloads");
if (portfolioDownloads) {
  const vueApp = createApp(DownloadsList);
  vueApp.component("font-awesome-icon", FontAwesomeIcon as Component);
  vueApp.mount(portfolioDownloads);
}

const social = document.getElementById("social");
if (social) {
  const title = social.getAttribute("property");
  const vueApp = createApp(SocialButtons, {
    title: title || "",
    url: window.location.href,
  });
  vueApp.component("font-awesome-icon", FontAwesomeIcon as Component);
  vueApp.mount(social);
}

const search = document.getElementById("siteSearch");
if (search) {
  const urlParams = new URLSearchParams(window.location.search);
  const q = urlParams.get("q");
  const vueApp = createApp(SearchView, {
    query: q || "",
  });
  vueApp.mount(search);
}

const userProfile = document.getElementById("userProfile");
if (userProfile) {
  const vueApp = createApp(ProfileView);
  vueApp.component("font-awesome-icon", FontAwesomeIcon as Component);
  vueApp.component("AppIcon", AppIcon);
  vueApp.mount(userProfile);
}

function decoratePageContent(root: ParentNode, exclude?: HTMLElement): () => void {
  const apps: VueApp[] = [];
  const select = (selector: string) =>
    Array.from(root.querySelectorAll(selector)).filter((el) => !exclude?.contains(el));

  select("i.icon[data-label]").forEach((x) => {
    const label = x.getAttribute("data-label");
    const type = x.getAttribute("datatype");
    const vueApp = createApp(AppIcon, { icon: label || "", lib: type || "" });
    vueApp.component("font-awesome-icon", FontAwesomeIcon as Component);
    vueApp.mount(x);
    apps.push(vueApp);
  });

  select("span.date[data-label]").forEach((x) => {
    const label = x.getAttribute("data-label");
    const text = x.textContent?.trim() ?? "";
    const vueApp = createApp(DateFormatter, {
      date: text,
      formatStr: label || "LL",
    });
    vueApp.mount(x);
    apps.push(vueApp);
  });

  function mountHighlighting(prefix: string, el: Element): void {
    if (!el.className.startsWith(prefix)) return;

    const lang = el.className.replace(prefix, "").replace(";", "").trim();
    const app = createApp(CodeHighlighter, {
      content: el.textContent ?? "",
      lang: lang,
    });
    el.textContent = "";
    app.mount(el);
    apps.push(app);
  }

  select("pre, code").forEach((el) => {
    mountHighlighting("brush: ", el);
    mountHighlighting("language-", el);
  });

  return () => apps.forEach((app) => app.unmount());
}

const blogNav = document.getElementById("blogNavigation");
const blogResults = document.querySelector<HTMLElement>("[data-blog-results]");
if (blogNav && blogResults) {
  const search = ref(window.location.search);
  createApp(() => h(BlogNavigation, { search: search.value })).mount(blogNav);

  decoratePageContent(document, blogResults);
  enableBlogNavigation(blogResults, decoratePageContent, (url, page) => {
    search.value = url.search;
    appTitle.value = page.getElementById("app")?.getAttribute("datafld") ?? "";
  });
} else {
  decoratePageContent(document);
}

const admin = document.getElementById("admin");
if (admin) {
  const router = createAdminRouter();
  const vueApp = createApp(AdminApp);
  vueApp.use(router);
  vueApp.use(Vue3ProgressPlugin);
  vueApp.component("font-awesome-icon", FontAwesomeIcon as Component);
  vueApp.mount(admin);
}
