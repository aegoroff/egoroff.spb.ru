package indie

import (
	"egoroff.spb.ru/app/auth"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"
	"github.com/dgrijalva/jwt-go"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"github.com/patrickmn/go-cache"
	"io/ioutil"
	"log"
	"net/http"
	"net/url"
	"strings"
	"time"
)

type IndieClaims struct {
	jwt.StandardClaims
	ClientID    string `json:"client_id"`
	RedirectURL string `json:"redirect_url"`
}

type Auth struct {
	cache *cache.Cache
}

func NewAuth() *Auth {
	c := cache.New(10*time.Minute, 10*time.Minute)
	return &Auth{cache: c}
}

func (a *Auth) Route(r *gin.Engine) {
	r.GET("/auth", a.get)
}

func (a *Auth) get(c *gin.Context) {
	admin := false
	authorized := false
	auth.IfAuthenticated(c, func(user *domain.User) {
		authorized = true
		admin = user.Admin
	})

	if !authorized {
		auth.UpdateSession(c, func(s sessions.Session) {
			s.Set(auth.RedirectUrlCookie, c.Request.Referer())
		})
		c.Redirect(http.StatusFound, "/login")
		return
	}

	if !admin {
		framework.Error401(c)
		return
	}

	clientId := c.Query("client_id")
	redirectUri := c.Query("redirect_uri")
	state := c.Query("state")

	if strings.HasPrefix(redirectUri, clientId) {
		code, _ := newJwt(IndieClaims{
			ClientID:    clientId,
			RedirectURL: redirectUri,
			StandardClaims: jwt.StandardClaims{
				Audience:  "",
				ExpiresAt: int64(10 * time.Minute),
				IssuedAt:  time.Now().Unix(),
				Issuer:    ME,
			},
		})

		q := &url.URL{
			RawQuery: url.Values{
				"state": {state},
				"code":  {code},
			}.Encode(),
		}

		a.cache.Set(code, code, cache.DefaultExpiration)

		c.Redirect(http.StatusFound, redirectUri+q.String())
	} else {
		// TODO: validate redirect here
		client := http.DefaultClient
		uri, err := url.Parse(clientId)
		if err != nil {
			c.AbortWithError(http.StatusBadRequest, err)
		}
		resp, _ := client.Get(uri.String())
		if resp != nil {
			bb, err := ioutil.ReadAll(resp.Body)
			bs := string(bb)
			if err != nil {
				log.Println(err)
			}
			log.Println(bs)
		}
	}
}

func (a *Auth) validateTokenReq(req tokenReq) bool {
	_, ok := a.cache.Get(req.Code)
	if !ok {
		return ok
	}
	defer a.cache.Delete(req.Code)

	t, err := decodeJwt(req.Code)
	if err != nil {
		log.Println(err)
		return false
	}
	err = t.Claims.Valid()
	if err != nil {
		log.Println(err)
		return false
	}

	claims, ok := t.Claims.(jwt.MapClaims)
	if !ok {
		log.Printf("Not jwt.MapClaims: %v", t.Claims)
		return ok
	}

	ok = claims["client_id"] == req.ClientId && claims["redirect_url"] == req.RedirectUri
	if !ok {
		log.Printf("claims.ClientID: %v\n", claims["client_id"])
		log.Println("req.ClientId: " + req.ClientId)
		log.Printf("claims.RedirectURL: %v\n", claims["redirect_url"])
		log.Println("req.RedirectUri: " + req.RedirectUri)
	}

	return ok
}
