<script lang="ts">
import 'reflect-metadata'
import axios from 'axios'
import { Vue } from 'vue-property-decorator'
import { Section } from '../components/Navigation.vue'
import { injectable } from 'vue-typescript-inject'
import { Archive } from '@/components/BlogNavigation.vue'
import { toQuery } from '@/util'
import { FullUserInfo } from '@/views/Profile.vue'
import { Post } from '@/components/admin/EditPost.vue'
import { Download } from '@/components/admin/EditDownload.vue'

export class ApiResult<T> {
  public status!: string
  public count!: number
  public page!: number
  public pages!: number
  public row!: string
  public result!: Array<T>
}

export class Query {
  public limit!: string
  public offset!: string
  public tag!: string
  public year!: string
  public month!: string
  public page!: string
}

export class Nav {
  public sections!: Array<Section>
  public breadcrumbs!: Array<Section>
}

export class User {
  public loginOrName!: string
  public provider!: string
  public authenticated!: boolean
  public admin!: boolean
}

@injectable()
export default class ApiService extends Vue {
  public getNavigation (): Nav {
    const navigation = new Nav()
    navigation.sections = new Array<Section>()
    navigation.breadcrumbs = new Array<Section>()

    const q = encodeURIComponent(document.location.pathname)
    axios.get<Nav>(`/api/v2/navigation/?uri=${q}`).then(r => {
      navigation.sections.push(...r.data.sections)
      if (r.data.breadcrumbs) {
        navigation.breadcrumbs.push(...r.data.breadcrumbs)
      }
    })

    return navigation
  }

  public async getBlogArchive (): Promise<Archive> {
    this.$Progress.start()
    return await axios.get<Archive>('/api/v2/blog/archive/').then(r => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }

  public async getUser (): Promise<User> {
    return await axios.get<User>('/api/v2/auth/user/').then(r => {
      return r.data
    })
  }

  public async getFullUserInfo (): Promise<FullUserInfo> {
    return await axios.get<FullUserInfo>('/api/v2/auth/userinfo/').then(r => {
      return r.data
    })
  }

  public updateFullUserInfo (u: FullUserInfo): void {
    axios.put<FullUserInfo>('/api/v2/auth/userinfo/', u)
  }

  public editPost (p: Post): void {
    axios.put<Post>('/api/v2/admin/post', p)
  }

  public deletePost (id: number): void {
    axios.delete(`/api/v2/admin/post/${id}`)
  }

  public editDownload (d: Download): void {
    axios.put<Download>('/api/v2/admin/download/', d)
  }

  public deleteDownload (id: number): void {
    axios.delete(`/api/v2/admin/download/${id}`)
  }

  public async getDownloads<T> (q?: Query): Promise<ApiResult<T>> {
    this.$Progress.start()
    return await axios.get<ApiResult<T>>(`/api/v2/admin/download/${toQuery(q)}`).then(r => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }

  public async getPosts<T> (q?: Query): Promise<ApiResult<T>> {
    this.$Progress.start()
    return await axios.get<ApiResult<T>>(`/api/v2/blog/posts/${toQuery(q)}`).then(r => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }

  public async getAdminPosts<T> (q?: Query): Promise<ApiResult<T>> {
    this.$Progress.start()
    return await axios.get<ApiResult<T>>(`/api/v2/admin/posts/${toQuery(q)}`).then(r => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }
}
</script>
