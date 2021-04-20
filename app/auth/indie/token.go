package indie

import (
	"fmt"
	"github.com/dgrijalva/jwt-go"
	"github.com/gin-gonic/gin"
	"golang.org/x/oauth2"
	"io/ioutil"
	"log"
	"net/http"
	"time"
)

type TokenEndpoint struct {
	auth *Auth
}

type tokenReq struct {
	GrantType   string `form:"grant_type"`
	Code        string `form:"code"`
	ClientId    string `form:"client_id"`
	RedirectUri string `form:"redirect_uri"`
	Me          string `form:"me"`
}

func NewTokenEndpoint(a *Auth) *TokenEndpoint {
	return &TokenEndpoint{auth: a}
}

func (t *TokenEndpoint) Route(r *gin.Engine) {
	r.POST("/token", t.token)
}

func (t *TokenEndpoint) token(c *gin.Context) {
	if c.ContentType() != "application/x-www-form-urlencoded" {
		c.Abort()
		return
	}
	var req tokenReq
	err := c.Bind(&req)
	if err != nil {
		log.Println(err)
		c.Abort()
		return
	}

	if !t.auth.validateCode(req.Code) {
		c.AbortWithError(http.StatusUnauthorized, fmt.Errorf("invalid code for: %s", req.ClientId))
		return
	}

	access, err := newJwt(IndieClaims{
		ClientID:    req.ClientId,
		RedirectURL: req.RedirectUri,
		StandardClaims: jwt.StandardClaims{
			IssuedAt: time.Now().Unix(),
			Issuer:   "egoroff.spb.ru",
		},
	})

	if err != nil {
		log.Println(err)
		c.AbortWithError(http.StatusUnauthorized, err)
		return
	}
	tok := oauth2.Token{
		AccessToken: access,
		TokenType:   "Bearer",
		Expiry:      time.Time{},
	}

	c.JSON(http.StatusOK, tok)
}

func newJwt(claims jwt.Claims) (string, error) {
	keyData, _ := ioutil.ReadFile("egoroffspbrupri.pem")
	key, _ := jwt.ParseRSAPrivateKeyFromPEM(keyData)

	method := jwt.GetSigningMethod("RS256")
	token := jwt.NewWithClaims(method, claims)
	return token.SignedString(key)
}

func verifyJwt(token string) bool {
	t, err := jwt.Parse(token, func(token *jwt.Token) (interface{}, error) {
		key, _ := ioutil.ReadFile("egoroffspbrupub.pem")
		return jwt.ParseRSAPublicKeyFromPEM(key)
	})
	if err != nil {
		log.Println(err)
		return false
	}
	err = t.Claims.Valid()
	if err != nil {
		log.Println(err)
		return false
	}
	return true
}
