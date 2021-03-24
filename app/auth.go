package app

import (
	"github.com/gin-gonic/gin"
	"github.com/zalando/gin-oauth2/google"
)

type Auth struct {
}

func NewAuth() *Auth {
	return &Auth{}
}

func (a *Auth) Route(r *gin.Engine) {
	r.GET("/login", google.LoginHandler)
	r.GET("/signin", google.LoginHandler)
}
