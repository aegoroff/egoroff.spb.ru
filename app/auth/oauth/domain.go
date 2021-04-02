package oauth

// Credentials stores client-ids.
type Credentials struct {
	ClientID     string `json:"clientid"`
	ClientSecret string `json:"secret"`
}

type AuthProvider struct {
	Name        string      `json:"name"`
	Credentials Credentials `json:"credentials"`
	RedirectURL string      `json:"redirect_url"`
	Scopes      []string    `json:"scopes"`
}
