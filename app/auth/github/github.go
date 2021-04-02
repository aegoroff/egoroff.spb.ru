// Package github provides you access to Github's OAuth2
// infrastructure.
package github

import (
	"context"
	"egoroff.spb.ru/app/auth/oauth"
	"egoroff.spb.ru/app/domain"
	"fmt"
	"net/http"

	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"github.com/google/go-github/github"
	"golang.org/x/oauth2"
	oauth2gh "golang.org/x/oauth2/github"
)

var (
	conf *oauth2.Config
)

const Provider = "github"

func Setup() {
	conf = oauth.NewConfig(Provider, oauth2gh.Endpoint)
}

func GetLoginURL(state string) string {
	return conf.AuthCodeURL(state)
}

func Auth() gin.HandlerFunc {
	return func(ctx *gin.Context) {
		var (
			user *github.User
		)

		// Handle the exchange code to initiate a transport.
		session := sessions.Default(ctx)

		retrievedState := session.Get("state")
		if retrievedState != ctx.Query("state") {
			ctx.AbortWithError(http.StatusUnauthorized, fmt.Errorf("Invalid session state: %s", retrievedState))
			return
		}

		tok, err := conf.Exchange(context.Background(), ctx.Query("code"))
		if err != nil {
			ctx.AbortWithError(http.StatusBadRequest, fmt.Errorf("Failed to do exchange: %v", err))
			return
		}
		client := github.NewClient(conf.Client(context.Background(), tok))
		user, _, err = client.Users.Get(context.Background(), "")
		if err != nil {
			ctx.AbortWithError(http.StatusBadRequest, fmt.Errorf("Failed to get user: %v", err))
			return
		}

		login := dereference(user.Login)
		u := domain.User{
			Active:      true,
			Email:       dereference(user.Email),
			FederatedId: fmt.Sprintf("github_%s", login),
			Name:        dereference(user.Name),
			Verified:    true,
			AvatarUrl:   dereference(user.AvatarURL),
			Username:    login,
		}
		ctx.Set("user", u)
	}
}

func dereference(s *string) string {
	result := ""
	if s != nil {
		result = *s
	}
	return result
}
