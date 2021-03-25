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

	callback := r.Group("/_s/callback/google/authorized/")
	callback.GET("/", a.callback)
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
	validator := google.Auth()
	validator(c)
	if c.IsAborted() {
		error401(c)
		return
	}
	user := c.MustGet("user").(google.User)
	log.Println("sub: " + user.Sub)
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
