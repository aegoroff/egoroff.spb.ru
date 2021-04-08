<script lang="ts">
import 'reflect-metadata'
import axios from 'axios'
import { Vue } from 'vue-property-decorator'
import { Section } from '../components/Navigation.vue'
import { injectable } from 'vue-typescript-inject'
import { Archive } from '@/components/BlogNavigation.vue'

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

  public getBreadcrumbs (): Array<Section> {
    const navigation = new Array<Section>()

    axios.post<Array<Section>>('/api/v2/breadcrumbs/', { uri: document.location.pathname }).then(r => {
      navigation.push(...r.data)
    })

    return navigation
  }
}
</script>
