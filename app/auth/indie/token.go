package indie

import (
	"fmt"
	"github.com/dgrijalva/jwt-go"
	"github.com/gin-gonic/gin"
	"io/ioutil"
	"log"
	"net/http"
	"strings"
	"time"
)

const SCOPES = "create media delete"
const ME = "https://www.egoroff.spb.ru/"

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

type Token struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
	Scope       string `json:"scope"`
	Me          string `json:"me"`
}

func NewTokenEndpoint(a *Auth) *TokenEndpoint {
	return &TokenEndpoint{auth: a}
}

func (t *TokenEndpoint) Route(r *gin.Engine) {
	r.POST("/token", t.tokenGenerate)
	r.GET("/token", t.tokenVerify)
}

func (t *TokenEndpoint) tokenVerify(c *gin.Context) {
	auth := c.GetHeader("Authorization")
	unauthorized := gin.H{
		"error": "unauthorized",
	}
	if auth == "" {
		c.AbortWithStatusJSON(http.StatusUnauthorized, unauthorized)
		return
	}
	token := strings.TrimPrefix(auth, "Bearer ")
	tokenD, err := decodeJwt(token)
	if err != nil {
		c.AbortWithStatusJSON(http.StatusUnauthorized, unauthorized)
		return
	}
	err = tokenD.Claims.Valid()
	if err != nil {
		c.AbortWithStatusJSON(http.StatusUnauthorized, unauthorized)
		return
	}
	claims, ok := tokenD.Claims.(IndieClaims)
	if !ok {
		log.Printf("Invalid claims: %v", tokenD.Claims)
		c.AbortWithStatusJSON(http.StatusUnauthorized, unauthorized)
		return
	}
	c.JSON(http.StatusOK, gin.H{
		"me":        claims.Issuer,
		"client_id": claims.ClientID,
		"scope":     SCOPES,
	})
}

func (t *TokenEndpoint) tokenGenerate(c *gin.Context) {
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
			Issuer:   ME,
		},
	})

	if err != nil {
		log.Println(err)
		c.AbortWithError(http.StatusUnauthorized, err)
		return
	}
	tok := Token{
		AccessToken: access,
		TokenType:   "Bearer",
		Scope:       SCOPES,
		Me:          ME,
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
	t, err := decodeJwt(token)
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

func decodeJwt(token string) (*jwt.Token, error) {
	t, err := jwt.Parse(token, func(token *jwt.Token) (interface{}, error) {
		key, _ := ioutil.ReadFile("egoroffspbrupub.pem")
		return jwt.ParseRSAPublicKeyFromPEM(key)
	})
	if err != nil {
		return nil, err
	}
	return t, nil
}
