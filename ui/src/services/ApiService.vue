<script lang="ts">
import 'reflect-metadata'
import axios from 'axios'
import { Vue } from 'vue-property-decorator'
import { Section } from '../components/Navigation.vue'
import { injectable } from 'vue-typescript-inject'

@injectable()
export default class ApiService extends Vue {
  public getNavigation (): Array<Section> {
    const navigation = new Array<Section>()

    this.$Progress.start()
    axios.get<Array<Section>>('/api/v2/navigation.json').then(r => {
      navigation.push(...r.data)
    }).finally(() => this.$Progress.finish())

    return navigation
  }
}
</script>
