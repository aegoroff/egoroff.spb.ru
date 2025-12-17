import { createApp, h } from "vue";
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
import FromNow from "@/components/FromNow.vue";
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
  const vueApp = createApp({
    render() {
      return h(App, { title: t || "" });
    },
  });
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.component("AppIcon", AppIcon);
  vueApp.component("DateFormatter", DateFormatter);
  vueApp.component("FromNow", FromNow);
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
  vueApp.use(VueSocialSharing);
  vueApp.config.globalProperties.emitter = emitter;

  vueApp.mount("#app");
}

if (document.getElementById("blogNavigation")) {
  const vueApp = createApp(BlogNavigation);
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#blogNavigation");
}

if (document.getElementById("blogcontainer") && window.location.hash) {
  const hash = window.location.hash.substring(1);
  const vueApp = createApp({
    render() {
      return h(BlogAnnounces, { q: hash });
    },
  });
  vueApp.component("DateFormatter", DateFormatter);
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#blogcontainer");

  const blogTitleElement = document.getElementById("blogSmallTitle");
  if (blogTitleElement) {
    const e = hash.split("=");
    let titleText = `все посты по метке: ${e[1]}`;

    const vueApp2 = createApp({
      render() {
        return h(BlogTitle, { text: titleText });
      },
    });
    vueApp2.mount("#blogSmallTitle");
  }
}

if (document.getElementById("portfolioDownloads")) {
  const vueApp = createApp(Downloads);
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#portfolioDownloads");
}

if (document.getElementById("social")) {
  const title = document.getElementById("social")?.getAttribute("property");
  const vueApp = createApp({
    render() {
      return h(Social, {
        title: title || "",
        url: window.location.href,
        networks: ["vk"],
      });
    },
  });
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#social");
}

if (document.getElementById("siteSearch")) {
  const apiKey = document
    .getElementById("siteSearch")
    ?.getAttribute("property");
  const cx = document.getElementById("siteSearch")?.getAttribute("datafld");
  const urlParams = new URLSearchParams(window.location.search);
  const q = urlParams.get("q");
  const vueApp = createApp({
    render() {
      return h(Search, {
        apiKey: apiKey || "",
        cx: cx || "",
        query: q || "",
      });
    },
  });
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#siteSearch");
}

if (document.getElementById("userProfile")) {
  const vueApp = createApp(Profile);
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.component("AppIcon", AppIcon);
  vueApp.mount("#userProfile");
}

const icons = document.querySelectorAll("i.icon[data-label]");
icons.forEach((x) => {
  const label = x.getAttribute("data-label");
  const type = x.getAttribute("datatype");
  const icon = label || "";
  const lib = type || "";
  const vueApp = createApp({
    render() {
      return h(AppIcon, {
        icon: icon,
        lib: lib,
      });
    },
  });
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount(x);
});

const dates = document.querySelectorAll("span.date[data-label]");
dates.forEach((x) => {
  const label = x.getAttribute("data-label");
  const fmt = label === null ? "LL" : label;
  const text = x.textContent?.trim() ?? "";
  if (fmt === "from-now") {
    const vueApp = createApp({
      render() {
        return h(FromNow, {
          date: text,
        });
      },
    });
    vueApp.mount(x);
  } else {
    const vueApp = createApp({
      render() {
        return h(DateFormatter, {
          date: text,
          formatStr: fmt,
        });
      },
    });
    vueApp.mount(x);
  }
});

function mountHighlighting(prefix: string, el: Element): void {
  if (!el.className.startsWith(prefix)) return;

  const lang = el.className.replace(prefix, "").replace(";", "").trim();

  const app = createApp(Highlighter, {
    content: el.textContent ?? "",
    lang: lang,
  });

  el.textContent = "";
  app.mount(el);
}

const snippets = document.querySelectorAll("pre, code");

snippets.forEach((el) => {
  mountHighlighting("brush: ", el);
  mountHighlighting("language-", el);
});

const alerts = document.querySelectorAll(".alert");
alerts.forEach((x) => {
  const type = x.getAttribute("data-label");
  const alert = type || "success";
  const vueApp = createApp({
    render() {
      return h(Alert, {
        content: x.textContent || "",
        type: alert,
      });
    },
  });
  vueApp.mount(x);
});

if (document.getElementById("admin")) {
  const router = createAdminRouter();
  const vueApp = createApp(AdminApp);
  vueApp.use(router);
  vueApp.component("font-awesome-icon", FontAwesomeIcon);
  vueApp.mount("#admin");
}
