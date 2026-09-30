export interface Archive {
  tags: Array<Tag>
  years: Array<Year>
}

export interface Month {
  month: number
  posts: number
}

export interface Year {
  year: number
  posts: number
  months: Array<Month>
}

export interface Tag {
  title: string
  level: number
}

/** Filters of the posts listing; the server always pages by a fixed size. */
export interface Query {
  tag?: string
  year?: string
  month?: string
  page?: string
}

/** Public post teaser (`SmallPost` on the server). */
export interface Post {
  Created: string
  id: number
  Title: string
  ShortText: string
}

/** Full post as the admin API reads and writes it (`Post` on the server). */
export interface EditablePost {
  Created: string
  Modified: string
  id: number
  Title: string
  IsPublic: boolean
  Markdown: boolean
  Tags: Array<string>
  Text: string
  ShortText: string
}

export const emptyPost = (): EditablePost => ({
  Created: "",
  Modified: "",
  id: 0,
  Title: "",
  IsPublic: false,
  Markdown: false,
  Tags: [],
  Text: "",
  ShortText: "",
});
