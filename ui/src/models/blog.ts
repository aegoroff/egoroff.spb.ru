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

export class Query {
  public limit!: string;
  public offset!: string;
  public tag!: string;
  public year!: string;
  public month!: string;
  public page!: string;
}

export class Post {
  public Key!: string
  public Created!: string
  public id!: number
  public Title!: string
  public ShortText!: string
}

export class EditablePost {
  public Created!: string;
  public id!: number;
  public Title!: string;
  public IsPublic!: boolean;
  public Markdown!: boolean;
  public Tags!: Array<string>;
  public Text!: string;
  public ShortText!: string;
}
