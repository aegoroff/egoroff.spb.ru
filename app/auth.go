package app

import (
	"crypto/rand"
	"encoding/base64"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"github.com/zalando/gin-oauth2/google"
	"log"
	"net/http"
	"time"
)

type Auth struct {
}

func NewAuth() *Auth {
	return &Auth{}
}

func (a *Auth) Route(r *gin.Engine) {
	r.GET("/login", a.signin)
	r.GET("/signin", a.signin)
	r.GET("/logout", a.signout)

	callback := r.Group("/_s/callback/google/authorized/")
	callback.GET("/", a.callback)
}

func (a *Auth) signout(c *gin.Context) {
	session := sessions.Default(c)
	session.Delete("user-sub")
	err := session.Save()
	if err != nil {
		log.Println(err)
	}
	c.Redirect(http.StatusFound, "/")
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
	user, ok := c.Get("user")
	if ok {
		u := user.(google.User)
		session := sessions.Default(c)
		session.Set("user-sub", u.Sub)
		err := session.Save()
		if err != nil {
			log.Println(err)
		}
		repo := NewRepository()
		existing, err := repo.UserByFederatedId(u.Sub)
		if existing == nil {
			err = repo.NewUser(&User{
				Model: Model{
					Created:  time.Now(),
					Modified: time.Now(),
				},
				Active:      true,
				Email:       u.Email,
				FederatedId: u.Sub,
				Name:        u.Name,
				Verified:    u.EmailVerified,
			})
		}
	}
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
