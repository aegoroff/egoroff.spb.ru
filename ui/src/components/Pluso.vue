<script lang="ts">
import { Vue } from 'vue-property-decorator'

export default class Pluso extends Vue {
  constructor () {
    super()
    const plusoScript = document.createElement('script')
    plusoScript.setAttribute('src', 'https://share.pluso.ru/pluso-like.js')
    plusoScript.setAttribute('async', 'true')
    document.head.appendChild(plusoScript)
  }
}
</script>
