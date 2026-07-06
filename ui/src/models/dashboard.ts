export class DashboardStats {
  public posts_count: number = 0;
  public downloads_count: number = 0;
  public users_count: number = 0;

  // Backward compatibility with the template
  get postsCount(): number {
    return this.posts_count || 0;
  }

  get downloadsCount(): number {
    return this.downloads_count || 0;
  }

  get usersCount(): number {
    return this.users_count || 0;
  }
}
