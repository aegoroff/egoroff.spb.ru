<template>
  <div class="col-lg-3" id="blogNavigation">
    <b-tabs>
      <b-tab title="Метки" active>
        <Tags v-bind:tags="archive.tags"/>
      </b-tab>
      <b-tab title="Архив">
        <Chrono v-bind:years="archive.years"/>
      </b-tab>
    </b-tabs>
  </div>
</template>

<script lang="ts">
import 'reflect-metadata'
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService from '@/services/ApiService.vue'
import Tags, { Tag } from '@/components/Tags.vue'
import Chrono, { Year } from '@/components/Chrono.vue'
import { inject } from 'vue-typescript-inject'

export class Archive {
  public tags!: Array<Tag>
  public years!: Array<Year>
}

@Component({
  components: {
    Tags,
    Chrono
  },
  providers: [ApiService]
})
export default class BlogNavigation extends Vue {
  @Prop() private archive: Archive
  @inject() private api!: ApiService

  constructor () {
    super()
    this.archive = new Archive()
  }

  mounted (): void {
    this.api.getBlogArchive().then(x => {
      this.archive = x
    })
  }
}
</script>
