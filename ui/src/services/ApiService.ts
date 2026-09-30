import axios from "axios";
import { useProgress } from "@marcoschulte/vue3-progress";
import { toQuery } from "@/util";
import type { Archive, EditablePost, Post, Query } from "@/models/blog";
import type { FullUserInfo, Nav, Section, User } from "@/models/common";
import type { DashboardStats } from "@/models/dashboard";
import type { Download, FilesContainer } from "@/models/portfolio";

/** One page of a server listing (`ApiResult` on the server). */
export interface ApiResult<T> {
  status: string;
  count: number;
  page: number;
  pages: number;
  result: Array<T>;
}

/** Body of a successful admin create, update or delete. */
interface OperationResult {
  result: string;
}

/** Navigation as the server sends it: both lists may be missing. */
interface NavResponse {
  sections?: Array<Section>;
  breadcrumbs?: Array<Section>;
}

async function get<R>(url: string): Promise<R> {
  const response = await axios.get<R>(url);
  return response.data;
}

/** GET that shows the page progress bar; used by admin screens. */
function getWithProgress<R>(url: string): Promise<R> {
  return useProgress().attach(get<R>(url));
}

class ApiService {
  /** Navigation for the current page; falls back to empty lists on failure. */
  public async getNavigation(): Promise<Nav> {
    const q = encodeURIComponent(document.location.pathname);
    try {
      const nav = await get<NavResponse>(`/api/v2/navigation/?uri=${q}`);
      return { sections: nav.sections ?? [], breadcrumbs: nav.breadcrumbs ?? [] };
    } catch (error) {
      console.error("Failed to fetch navigation:", error);
      return { sections: [], breadcrumbs: [] };
    }
  }

  public getBlogArchive(): Promise<Archive> {
    return get<Archive>("/api/v2/blog/archive/");
  }

  public getUser(): Promise<User> {
    return get<User>("/api/v2/auth/user/");
  }

  public getFullUserInfo(): Promise<FullUserInfo> {
    return get<FullUserInfo>("/api/v2/auth/userinfo/");
  }

  public getPosts(q?: Query): Promise<ApiResult<Post>> {
    return get<ApiResult<Post>>(`/api/v2/blog/posts/${toQuery(q)}`);
  }

  public getDownloadableFiles(): Promise<ApiResult<FilesContainer>> {
    return get<ApiResult<FilesContainer>>("/api/v2/portfolio/files/");
  }

  public getAdminPosts(q?: Query): Promise<ApiResult<EditablePost>> {
    return getWithProgress<ApiResult<EditablePost>>(`/api/v2/admin/posts/${toQuery(q)}`);
  }

  public getDownloads(q?: Query): Promise<ApiResult<Download>> {
    return getWithProgress<ApiResult<Download>>(`/api/v2/admin/download/${toQuery(q)}`);
  }

  public getDashboardStats(): Promise<DashboardStats> {
    return getWithProgress<DashboardStats>("/api/v2/admin/dashboard/");
  }

  public getUsers(): Promise<ApiResult<FullUserInfo>> {
    return getWithProgress<ApiResult<FullUserInfo>>("/api/v2/admin/users/");
  }

  public async createPost(p: EditablePost): Promise<void> {
    await axios.post<OperationResult>("/api/v2/admin/post", p);
  }

  public async editPost(p: EditablePost): Promise<void> {
    await axios.put<OperationResult>("/api/v2/admin/post", p);
  }

  public async deletePost(id: number): Promise<void> {
    await axios.delete<OperationResult>(`/api/v2/admin/post/${id}`);
  }

  public async editDownload(d: Download): Promise<void> {
    await axios.put<OperationResult>("/api/v2/admin/download/", d);
  }

  public async deleteDownload(id: number): Promise<void> {
    await axios.delete<OperationResult>(`/api/v2/admin/download/${id}`);
  }
}

export default ApiService;
