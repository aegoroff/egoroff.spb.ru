package app

import (
	"crypto/rand"
	"encoding/base64"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"github.com/zalando/gin-oauth2/google"
	"log"
	"net/http"
)

type Auth struct {
}

func NewAuth() *Auth {
	return &Auth{}
}

func (a *Auth) Route(r *gin.Engine) {
	r.GET("/login", a.signin)
	r.GET("/signin", a.signin)
	r.GET("/_s/callback/google/authorized/", a.callback).Use(google.Auth())
}

func (a *Auth) signin(c *gin.Context) {
	ctx := NewContext(c)
	state := randToken()
	session := sessions.Default(c)
	session.Set("state", state)
	err := session.Save()
	if err != nil {
		log.Println(err)
	}

	ctx["title"] = "Авторизация"
	ctx["google_signin_url"] = google.GetLoginURL(state)

	c.HTML(http.StatusOK, "signin.html", ctx)
}

func (a *Auth) callback(c *gin.Context) {
	log.Println(c.MustGet("user").(google.User))
	c.Redirect(http.StatusFound, "/")
}

func randToken() string {
	b := make([]byte, 32)
	_, err := rand.Read(b)
	if err != nil {
		log.Println(err)
	}
	return base64.StdEncoding.EncodeToString(b)
}
