export class Archive {
  public tags!: Array<Tag>
  public years!: Array<Year>
}

export class Month {
  public month!: number
  public posts!: number
}

export class Year {
  public year!: number
  public posts!: number
  public months!: Array<Month>
}

export class Tag {
  public title!: string
  public level!: number
}

export class Post {
  public Key!: string
  public Created!: string
  public id!: number
  public Title!: string
  public ShortText!: string
}
