import axios from 'axios'
import { Section } from '../components/Navigation.vue'
import { Archive } from '@/components/BlogNavigation.vue'
import { toQuery } from '@/util'
import { FullUserInfo } from '@/views/Profile.vue'
import { Post } from '@/components/admin/EditPost.vue'
import { Download } from '@/components/admin/EditDownload.vue'

export class ApiResult<T> {
  public status!: string
  public count!: number
  public page!: number
  public pages!: number
  public row!: string
  public result!: Array<T>
}

export class Query {
  public limit: string = '10'
  public offset: string = '0'
  public tag: string = ''
  public year: string = ''
  public month: string = ''
  public page: string = '1'
}

export class Nav {
  public sections!: Array<Section>
  public breadcrumbs!: Array<Section>
}

export class User {
  public loginOrName!: string
  public provider!: string
  public authenticated!: boolean
  public admin!: boolean
}

class ApiService {
  public getNavigation (): Nav {
    const navigation = new Nav()
    navigation.sections = []
    navigation.breadcrumbs = []

    const q = encodeURIComponent(document.location.pathname)
    axios.get<Nav>(`/api/v2/navigation/?uri=${q}`).then((response) => {
      navigation.sections.push(...response.data.sections)
      if (response.data.breadcrumbs) {
        navigation.breadcrumbs.push(...response.data.breadcrumbs)
      }
    }).catch(error => {
      console.error('Failed to fetch navigation:', error)
    })

    return navigation
  }

  public async getBlogArchive (): Promise<Archive> {
    try {
      const response = await axios.get<Archive>('/api/v2/blog/archive/')
      return response.data
    } catch (error) {
      console.error('Failed to fetch blog archive:', error)
      throw error
    }
  }

  public async getUser (): Promise<User> {
    try {
      const response = await axios.get<User>('/api/v2/auth/user/')
      return response.data
    } catch (error) {
      console.error('Failed to fetch user:', error)
      throw error
    }
  }

  public async getFullUserInfo (): Promise<FullUserInfo> {
    try {
      const response = await axios.get<FullUserInfo>('/api/v2/auth/userinfo/')
      return response.data
    } catch (error) {
      console.error('Failed to fetch full user info:', error)
      throw error
    }
  }

  public updateFullUserInfo (u: FullUserInfo): void {
    axios.put<FullUserInfo>('/api/v2/auth/userinfo/', u)
      .then(() => {
        console.log('User info updated successfully')
      })
      .catch(error => {
        console.error('Failed to update user info:', error)
      })
  }

  public editPost (p: Post): void {
    axios.put<Post>('/api/v2/admin/post', p)
      .then(() => {
        console.log('Post updated successfully')
      })
      .catch(error => {
        console.error('Failed to update post:', error)
      })
  }

  public deletePost (id: number): void {
    axios.delete(`/api/v2/admin/post/${id}`)
      .then(() => {
        console.log('Post deleted successfully')
      })
      .catch(error => {
        console.error('Failed to delete post:', error)
      })
  }

  public editDownload (d: Download): void {
    axios.put<Download>('/api/v2/admin/download/', d)
      .then(() => {
        console.log('Download updated successfully')
      })
      .catch(error => {
        console.error('Failed to update download:', error)
      })
  }

  public deleteDownload (id: number): void {
    axios.delete(`/api/v2/admin/download/${id}`)
      .then(() => {
        console.log('Download deleted successfully')
      })
      .catch(error => {
        console.error('Failed to delete download:', error)
      })
  }

  public async getDownloads<T> (q?: Query): Promise<ApiResult<T>> {
    try {
      const response = await axios.get<ApiResult<T>>(`/api/v2/admin/download/${toQuery(q)}`)
      return response.data
    } catch (error) {
      console.error('Failed to fetch downloads:', error)
      throw error
    }
  }

  public async getPosts<T> (q?: Query): Promise<ApiResult<T>> {
    try {
      const response = await axios.get<ApiResult<T>>(`/api/v2/blog/posts/${toQuery(q)}`)
      return response.data
    } catch (error) {
      console.error('Failed to fetch posts:', error)
      throw error
    }
  }

  public async getDownloadableFiles<T> (): Promise<ApiResult<T>> {
    try {
      const response = await axios.get<ApiResult<T>>('/api/v2/portfolio/files/')
      return response.data
    } catch (error) {
      console.error('Failed to fetch downloadable files:', error)
      throw error
    }
  }

  public async getAdminPosts<T> (q?: Query): Promise<ApiResult<T>> {
    try {
      const response = await axios.get<ApiResult<T>>(`/api/v2/admin/posts/${toQuery(q)}`)
      return response.data
    } catch (error) {
      console.error('Failed to fetch admin posts:', error)
      throw error
    }
  }
}

export default ApiService