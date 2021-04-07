<script lang="ts">
import { Vue } from 'vue-property-decorator'

export default class Pluso extends Vue {
  constructor () {
    super()
    const plusoScript = document.createElement('script')
    const src = (window.location.protocol === 'https:' ? 'https' : 'http') + '://share.pluso.ru/pluso-like.js'
    plusoScript.setAttribute('src', src)
    plusoScript.setAttribute('async', 'true')
    plusoScript.setAttribute('type', 'text/javascript')
    plusoScript.setAttribute('charset', 'UTF-8')
    document.body.appendChild(plusoScript)
  }
}
</script>
