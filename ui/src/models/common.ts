export class User {
  public loginOrName!: string;
  public provider!: string;
  public authenticated!: boolean;
  public admin!: boolean;
}

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

export class Section {
  public id!: string
  public title!: string
  public class!: string
  public icon!: string
  public active!: boolean
  public descr!: string
}

export class Nav {
  public sections!: Array<Section>;
  public breadcrumbs!: Array<Section>;
}