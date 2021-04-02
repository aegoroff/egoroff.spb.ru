package facebook

import (
	"context"
	"egoroff.spb.ru/app/auth/oauth"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/lib"
	"encoding/json"
	"fmt"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"github.com/golang/glog"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/facebook"
	"io/ioutil"
	"net/http"
)

const Provider = "facebook"

// User is a retrieved and authenticated user.
type User struct {
	Id    string `json:"id"`
	Name  string `json:"name"`
	Email string `json:"email"`
	Login string `json:"username"`
}

var conf *oauth2.Config

// Setup the authorization path
func Setup() {
	conf = oauth.NewConfig(Provider, facebook.Endpoint)
}

func GetLoginURL(state string) string {
	return conf.AuthCodeURL(state)
}

func Auth() gin.HandlerFunc {
	return func(ctx *gin.Context) {
		// Handle the exchange code to initiate a transport.
		session := sessions.Default(ctx)
		retrievedState := session.Get("state")
		if retrievedState != ctx.Query("state") {
			ctx.AbortWithError(http.StatusUnauthorized, fmt.Errorf("Invalid session state: %s", retrievedState))
			return
		}

		tok, err := conf.Exchange(context.Background(), ctx.Query("code"))
		if err != nil {
			ctx.AbortWithError(http.StatusBadRequest, err)
			return
		}

		client := conf.Client(context.Background(), tok)
		resp, err := client.Get("https://graph.facebook.com/me")
		if err != nil {
			ctx.AbortWithError(http.StatusBadRequest, err)
			return
		}
		defer lib.Close(resp.Body)
		data, err := ioutil.ReadAll(resp.Body)
		if err != nil {
			glog.Errorf("[Gin-OAuth] Could not read Body: %s", err)
			ctx.AbortWithError(http.StatusInternalServerError, err)
			return
		}

		var user User
		err = json.Unmarshal(data, &user)
		if err != nil {
			glog.Errorf("[Gin-OAuth] Unmarshal userinfo failed: %s", err)
			ctx.AbortWithError(http.StatusInternalServerError, err)
			return
		}
		// save userinfo, which could be used in Handlers

		u := domain.User{
			Active:      true,
			Email:       user.Email,
			FederatedId: user.Id,
			Name:        user.Name,
			Verified:    true,
			Username:    user.Login,
		}
		ctx.Set("user", u)
	}
}
