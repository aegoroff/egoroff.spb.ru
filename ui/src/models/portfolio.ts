export interface Downloadable {
  Title: string
  Path: string
  FileName: string
  Blake3Hash: string
  Size: number
}

export interface FilesContainer {
  Title: string
  Files: Array<Downloadable>
}

export interface Download {
  id: number
  title: string
}

export const emptyDownload = (): Download => ({ id: 0, title: "" });
