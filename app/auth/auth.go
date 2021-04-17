package auth

import (
	"egoroff.spb.ru/app/auth/facebook"
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

	callbackFb := r.Group("/_s/callback/facebook/authorized/")
	callbackFb.GET("/", a.callbackFacebook)
}

func (*Auth) signout(c *gin.Context) {
	updateSession(c, func(s sessions.Session) {
		s.Delete(framework.UserIdCookie)
	})

	c.Redirect(http.StatusFound, c.Request.Referer())
}

func (*Auth) signin(c *gin.Context) {
	ctx := framework.NewContext(c)
	state := randToken()

	updateSession(c, func(s sessions.Session) {
		s.Set(authStateCookie, state)
		s.Set(redirectUrlCookie, c.Request.Referer())
	})

	ctx["title"] = "Авторизация"
	ctx["google_signin_url"] = google.GetLoginURL(state)
	ctx["github_signin_url"] = github.GetLoginURL(state)
	ctx["facebook_signin_url"] = facebook.GetLoginURL(state)

	c.HTML(http.StatusOK, "signin.html", ctx)
}

func (a *Auth) callbackGoogle(c *gin.Context) {
	a.callback(c, google.Auth(), "google")
}

func (a *Auth) callbackGithub(c *gin.Context) {
	a.callback(c, github.Auth(), "github")
}

func (a *Auth) callbackFacebook(c *gin.Context) {
	a.callback(c, facebook.Auth(), "facebook")
}

func (a *Auth) callback(c *gin.Context, validator gin.HandlerFunc, provider string) {
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
			u.Provider = provider
			err = repo.NewUser(&u)
			if err != nil {
				log.Println(err)
			}
		} else {
			existing.Provider = provider
			err = repo.UpdateUser(existing, existing.Key)
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

func IfAuthenticated(c *gin.Context, success func(user *domain.User)) {
	sub := sessions.Default(c).Get(framework.UserIdCookie)
	if sub != nil {
		repo := db.NewRepository()
		u, err := repo.UserByFederatedId(sub.(string))
		if err != nil {
			log.Println(err)
		} else {
			success(u)
		}
	}
}
