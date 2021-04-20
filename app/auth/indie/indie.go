package indie

import (
	"github.com/gin-gonic/gin"
	"golang.org/x/oauth2"
	"net/http"
	"strings"
)

// Endpoint is egoroff.spb.ru OAuth 2.0 IndieAuth endpoint.
var Endpoint = oauth2.Endpoint{
	AuthURL:  "https://www.egoroff.spb.ru/auth",
	TokenURL: "https://www.egoroff.spb.ru/token",
}

func IndieAuth() gin.HandlerFunc {
	return func(c *gin.Context) {
		auth := c.GetHeader("Authorization")
		unauthorized := gin.H{
			"error": "unauthorized",
		}
		if auth == "" || strings.TrimSpace(auth) == "Bearer" {
			tok, _ := c.GetPostForm("access_token")
			if tok == "" {
				c.AbortWithStatusJSON(http.StatusUnauthorized, unauthorized)
				return
			}

			auth = "Bearer " + tok
		}
		token := strings.TrimPrefix(auth, "Bearer ")
		if !verifyJwt(token) {
			c.AbortWithStatusJSON(http.StatusUnauthorized, unauthorized)
		}
	}
}
