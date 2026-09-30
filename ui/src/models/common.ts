/** Current session user (`AuthorizedUser` on the server). */
export interface User {
  loginOrName: string
  provider: string
  authenticated: boolean
  admin: boolean
}

/** Stored user (`User` on the server). */
export interface FullUserInfo {
  admin: boolean
  created: string
  avatarUrl: string
  email: string
  name: string
  username: string
  federatedId: string
  verified: boolean
  provider: string
}

/** Site map section (`SiteSection` on the server). */
export interface Section {
  id: string
  title: string
  icon: string
  descr?: string
  keywords?: string
  active?: boolean
}

export interface Nav {
  sections: Array<Section>
  breadcrumbs: Array<Section>
}
