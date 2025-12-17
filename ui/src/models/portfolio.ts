export class Downloadable {
  public Title!: string
  public Path!: string
  public FileName!: string
  public Blake3Hash!: string
  public Size!: number
}

export class FilesContainer {
  public Title!: string
  public Files!: Array<Downloadable>
}

export class Download {
  public id!: number;
  public title!: string;
}