<script lang="ts">
import 'reflect-metadata'
import axios from 'axios'
import { Vue } from 'vue-property-decorator'
import { Section } from '../components/Navigation.vue'
import { injectable } from 'vue-typescript-inject'
import { Archive } from '@/components/BlogNavigation.vue'
import { Post } from '@/components/BlogAnnounces.vue'

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

@injectable()
export default class ApiService extends Vue {
  public getNavigation (): Nav {
    const navigation = new Nav()
    navigation.sections = new Array<Section>()
    navigation.breadcrumbs = new Array<Section>()

    const q = encodeURIComponent(document.location.pathname)
    axios.get<Nav>('/api/v2/navigation/?uri=' + q).then(r => {
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

  public async getPosts (q?: Query): Promise<ApiResult> {
    this.$Progress.start()
    return await axios.get<ApiResult>('/api/v2/blog/posts/' + this.toQuery(q)).then(r => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }

  toQuery (q?: Query): string {
    if (q === undefined) {
      return ''
    }
    let str = '?'
    for (const key in q) {
      if (str !== '?') {
        str += '&'
      }
      const v = Reflect.get(q, key)
      str += key + '=' + encodeURIComponent(v)
    }
    return str
  }
}
</script>
