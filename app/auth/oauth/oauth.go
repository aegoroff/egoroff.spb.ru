package oauth

import (
	"egoroff.spb.ru/app/db"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"golang.org/x/oauth2"
)

var (
	store sessions.CookieStore
)

func NewStore(secret []byte) {
	store = sessions.NewCookieStore(secret)
}

func NewConfig(name string, endpoint oauth2.Endpoint) *oauth2.Config {
	repo := db.NewRepository()
	provider := repo.Auth(name)
	conf := &oauth2.Config{
		ClientID:     provider.ClientID,
		ClientSecret: provider.ClientSecret,
		RedirectURL:  provider.RedirectURL,
		Scopes:       provider.Scopes,
		Endpoint:     endpoint,
	}
	return conf
}

func Session(name string) gin.HandlerFunc {
	return sessions.Sessions(name, store)
}
