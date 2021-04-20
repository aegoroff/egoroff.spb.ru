package indie

import (
	"golang.org/x/oauth2"
)

// Endpoint is egoroff.spb.ru OAuth 2.0 IndieAuth endpoint.
var Endpoint = oauth2.Endpoint{
	AuthURL:  "https://www.egoroff.spb.ru/auth",
	TokenURL: "https://www.egoroff.spb.ru/token",
}

func newIndieConfig(clientId string, redirectUrl string) *oauth2.Config {
	conf := &oauth2.Config{
		ClientID:    clientId,
		RedirectURL: redirectUrl,
		Scopes:      []string{"create", "media", "delete", "update"},
		Endpoint:    Endpoint,
	}
	return conf
}
