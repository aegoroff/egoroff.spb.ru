package auth

import (
	"egoroff.spb.ru/app/auth/github"
	"egoroff.spb.ru/app/auth/google"
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"

	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"time"
)

const redirectUrlCookie = "redirect_url_after_signin"
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
	callback.GET("/", a.callbackGoogle)

	callbackGh := r.Group("/_s/callback/github/authorized/")
	callbackGh.GET("/", a.callbackGithub)
}

func (a *Auth) signout(c *gin.Context) {
	updateSession(c, func(s sessions.Session) {
		s.Delete(framework.UserIdCookie)
	})

	c.Redirect(http.StatusFound, c.Request.Referer())
}

func (a *Auth) signin(c *gin.Context) {
	ctx := framework.NewContext(c)
	state := randToken()

	updateSession(c, func(s sessions.Session) {
		s.Set(authStateCookie, state)
		s.Set(redirectUrlCookie, c.Request.Referer())
	})

	ctx["title"] = "Авторизация"
	ctx["google_signin_url"] = google.GetLoginURL(state)
	ctx["github_signin_url"] = github.GetLoginURL(state)

	c.HTML(http.StatusOK, "signin.html", ctx)
}

func (a *Auth) callbackGoogle(c *gin.Context) {
	a.callback(c, google.Auth())
}

func (a *Auth) callbackGithub(c *gin.Context) {
	a.callback(c, github.Auth())
}

func (a *Auth) callback(c *gin.Context, validator gin.HandlerFunc) {
	validator(c)
	if c.IsAborted() {
		framework.Error401(c)
		return
	}
	user, ok := c.Get("user")
	if ok {
		u := user.(domain.User)

		updateSession(c, func(s sessions.Session) {
			s.Set(framework.UserIdCookie, u.FederatedId)
		})

		repo := db.NewRepository()
		existing, err := repo.UserByFederatedId(u.FederatedId)
		if err != nil {
			log.Println(err)
		}
		if existing == nil {
			u.Model = domain.Model{
				Created:  time.Now(),
				Modified: time.Now(),
			}
			err = repo.NewUser(&u)
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
