<script lang="ts">
import 'reflect-metadata'
import axios from 'axios'
import { Vue } from 'vue-property-decorator'
import { Section } from '../components/Navigation.vue'
import { injectable } from 'vue-typescript-inject'
import { Archive } from '@/components/BlogNavigation.vue'
import { Post } from '@/components/BlogAnnounces.vue'
import { toQuery } from '@/util'

export class ApiResult {
  public status!: string
  public count!: number
  public page!: number
  public pages!: number
  public row!: string
  public result!: Array<Post>
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
  public authenticated!: boolean
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
      navigation.breadcrumbs.push(...r.data.breadcrumbs)
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

  public async getPosts (q?: Query): Promise<ApiResult> {
    this.$Progress.start()
    return await axios.get<ApiResult>(`/api/v2/blog/posts/${toQuery(q)}`).then(r => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }
}
</script>
