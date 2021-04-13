<template>
  <b-container class="container" id="userProfile" fluid="lg">
    <div class="pb-2 mt-4 mb-2 border-bottom">
      <h1>
        <small class="float-right" title="Connected Accounts">
          <AppIcon lib="fab" v-bind:icon="user.provider"></AppIcon>
        </small>
        {{ user.name }}
      </h1>
    </div>

    <b-form @submit.prevent="update">
      <b-row>
        <b-col>
          <b-form-group id="login-group" label="Логин:" label-for="login">
            <b-form-input id="login" readonly v-model="user.username"></b-form-input>
          </b-form-group>
          <b-form-group id="name-group" label="Имя:" label-for="userName">
            <b-form-input id="userName" required v-model="user.name"></b-form-input>
          </b-form-group>

          <b-form-group
            id="email-group"
            label="Электропочта:"
            label-for="userEmail"
            description="С вашей электропочтой я никогда и ни с кем не поделюсь"
          >
            <b-form-input
              id="userEmail"
              v-model="user.email"
              type="email"
              placeholder="Введите адрес электропочты"
              required
            ></b-form-input>
          </b-form-group>
        </b-col>

        <b-col>
          <b-form-group id="avatar-group" label="Аватар:" label-for="avatar" v-if="user.avatarUrl">
            <b-img thumbnail id="avatar" fluid v-bind:src="user.avatarUrl" width="180">
            </b-img>
            <p class="text-muted">
              Изменить на
              <a href="//gravatar.com" target="_blank">Gravatar</a>
            </p>
          </b-form-group>
          <b-form-group id="avatar-group-add" label="Аватар:" label-for="newAvatarUrl" v-if="!user.avatarUrl">
            <b-form-input id="newAvatarUrl" v-model="newAvatarUrl"></b-form-input>
          </b-form-group>
        </b-col>
      </b-row>
      <b-row>
        <b-col>
          <b-button size="lg" type="submit" variant="primary">Обновить профиль</b-button>
        </b-col>
        <b-col></b-col>
      </b-row>
    </b-form>

  </b-container>
</template>

<script lang="ts">
import 'reflect-metadata'
import { Component, Prop, Vue } from 'vue-property-decorator'
import ApiService from '@/services/ApiService.vue'
import { inject } from 'vue-typescript-inject'
import AppIcon from '@/components/AppIcon.vue'

export class FullUserInfo {
  public id!: number
  public admin!: boolean
  public created!: string
  public avatarUrl!: string
  public email!: string
  public name!: string
  public username!: string
  public verified!: boolean
  public provider!: string
}

@Component({
  components: { AppIcon },
  providers: [ApiService]
})
export default class Profile extends Vue {
  @Prop() private user!: FullUserInfo
  @Prop() private newAvatarUrl!: string
  @inject() private api!: ApiService

  constructor () {
    super()
    this.user = new FullUserInfo()
  }

  mounted (): void {
    this.readProfile()
  }

  private readProfile () {
    this.api.getFullUserInfo().then(x => {
      this.user = x
    })
  }

  update (): void {
    if (this.newAvatarUrl) {
      this.user.avatarUrl = this.newAvatarUrl
    }
    this.api.updateFullUserInfo(this.user)
  }
}
</script>

<style scoped>

</style>
