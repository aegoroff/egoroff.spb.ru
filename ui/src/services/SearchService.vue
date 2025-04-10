<script lang="ts">
import 'reflect-metadata'
import { injectable } from 'vue-typescript-inject'
import { Vue } from 'vue-property-decorator'
import axios from 'axios'
import { toQuery } from '@/util'

export class Url {
  public type!: string
  public template!: string
}

export class SearchRequest {
  public title!: string
  public totalResults!: string
  public searchTerms!: string
  public count!: number
  public startIndex!: number
  public inputEncoding!: string
  public outputEncoding!: string
  public safe!: string
  public cx!: string
  public sort!: string
  public filter!: string
  public gl!: string
  public cr!: string
  public googleHost!: string
  public disableCnTwTranslation!: string
  public hq!: string
  public hl!: string
  public siteSearch!: string
  public siteSearchFilter!: string
  public exactTerms!: string
  public excludeTerms!: string
  public linkSite!: string
  public orTerms!: string
  public relatedSite!: string
  public dateRestrict!: string
  public lowRange!: string
  public highRange!: string
  public fileType!: string
  public rights!: string
  public searchType!: string
  public imgSize!: string
  public imgType!: string
  public imgColorType!: string
  public imgDominantColor!: string
}

export class Queries {
  public previousPage!: Array<SearchRequest>
  public request!: Array<SearchRequest>
  public nextPage!: Array<SearchRequest>
}

export class Context {
  public title!: string
}

export class SearchInfo {
  public searchTime!: number
  public formattedSearchTime!: string
  public totalResults!: string
  public formattedTotalResults!: string
}

export class Image {
  public contextLink!: string
  public height!: number
  public width!: number
  public byteSize!: number
  public thumbnailLink!: string
  public thumbnailHeight!: number
  public thumbnailWidth!: number
}

export class Label {
  public name!: string
  public displayName!: string
  // eslint-disable-next-line camelcase
  public label_with_op!: string
}

export class SearchResult {
  public kind!: string
  public title!: string
  public htmlTitle!: string
  public link!: string
  public displayLink!: string
  public snippet!: string
  public htmlSnippet!: string
  public cacheId!: string
  public formattedUrl!: string
  public htmlFormattedUrl!: string
  public pagemap!: Record<string, unknown>
  public mime!: string
  public fileFormat!: string
  public image!: Image
  public labels!: Array<Label>
}

export class Spelling {
  public correctedQuery!: string
  public htmlCorrectedQuery!: string
}

export class GoogleSearch {
  public kind!: string
  public url!: Url
  public queries!: Queries
  public context!: Context
  public searchInformation!: SearchInfo
  public spelling!: Spelling
  public items!: Array<SearchResult>
}

export class SearchQuery {
  public q!: string
  public key!: string
  public cx!: string
  public start!: number
}

@injectable()
export default class SearchService extends Vue {
  public async search (q?: SearchQuery): Promise<GoogleSearch> {
    this.$Progress.start()
    return await axios.get<GoogleSearch>(`https://www.googleapis.com/customsearch/v1${toQuery(q)}`).then((r: any) => {
      return r.data
    }).finally(() => this.$Progress.finish())
  }
}
</script>
