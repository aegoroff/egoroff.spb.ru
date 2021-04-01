package auth

type GithubUserInfo struct {
	Login      string `json:"login"`
	Id         int64  `json:"id"`
	NodeId     string `json:"node_id"`
	AvatarUrl  string `json:"avatar_url"`
	GravatarId string `json:"gravatar_id"`
	Url        string `json:"url"`
	HtmlUrl    string `json:"html_url"`
	Name       string `json:"name"`
	Company    string `json:"company"`
	Blog       string `json:"blog"`
	Location   string `json:"location"`
}
