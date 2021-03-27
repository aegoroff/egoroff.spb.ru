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

const redirectUrlCookie = "redirect_url_after_signin"
const userIdCookie = "user-sub"
const authStateCookie = "state"

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
	updateSession(c, func(s sessions.Session) {
		s.Delete(userIdCookie)
	})

	c.Redirect(http.StatusFound, c.Request.Referer())
}

func (a *Auth) signin(c *gin.Context) {
	ctx := NewContext(c)
	state := randToken()

	updateSession(c, func(s sessions.Session) {
		s.Set(authStateCookie, state)
		s.Set(redirectUrlCookie, c.Request.Referer())
	})

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

		updateSession(c, func(s sessions.Session) {
			s.Set(userIdCookie, u.Sub)
		})

		repo := NewRepository()
		existing, err := repo.UserByFederatedId(u.Sub)
		if err != nil {
			log.Println(err)
		}
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
			if err != nil {
				log.Println(err)
			}
		}
	}
	redirectUri := sessions.Default(c).Get(redirectUrlCookie)
	if redirectUri != nil {
		c.Redirect(http.StatusFound, redirectUri.(string))
	} else {
		c.Redirect(http.StatusFound, "/")
	}
}

func randToken() string {
	b := make([]byte, 32)
	_, err := rand.Read(b)
	if err != nil {
		log.Println(err)
	}
	return base64.StdEncoding.EncodeToString(b)
}
