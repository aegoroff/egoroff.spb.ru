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