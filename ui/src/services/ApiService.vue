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

@injectable()
export default class ApiService extends Vue {
  public getNavigation (): Array<Section> {
    const navigation = new Array<Section>()

    axios.get<Array<Section>>('/api/v2/navigation/').then(r => {
      navigation.push(...r.data)
    })

    return navigation
  }

  public async getBlogArchive (): Promise<Archive> {
    this.$Progress.start()
    return await axios.get<Archive>('/api/v2/blog/archive/').then(r => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }

  public async getPosts (): Promise<ApiResult> {
    this.$Progress.start()
    return await axios.get<ApiResult>('/api/v2/blog/posts/').then(r => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }

  public getBreadcrumbs (): Array<Section> {
    const navigation = new Array<Section>()

    axios.post<Array<Section>>('/api/v2/breadcrumbs/', { uri: document.location.pathname }).then(r => {
      navigation.push(...r.data)
    })

    return navigation
  }
}
</script>
