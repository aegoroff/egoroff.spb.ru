package app

import (
	"crypto/rand"
	"encoding/base64"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"github.com/zalando/gin-oauth2/google"
	"net/http"
)

type Auth struct {
	state string
}

func NewAuth() *Auth {
	return &Auth{}
}

func (a *Auth) Route(r *gin.Engine) {
	r.GET("/login", a.signin)
	r.GET("/signin", a.signin)
}

func (a *Auth) signin(c *gin.Context) {
	ctx := NewContext(c)
	a.state = randToken()
	session := sessions.Default(c)
	session.Set("state", a.state)
	session.Save()

	ctx["title"] = "Авторизация"
	ctx["google_signin_url"] = google.GetLoginURL(a.state)

	c.HTML(http.StatusOK, "signin.html", ctx)
}

func randToken() string {
	b := make([]byte, 32)
	rand.Read(b)
	return base64.StdEncoding.EncodeToString(b)
}
