package indie

import (
	"github.com/gin-gonic/gin"
	"golang.org/x/oauth2"
	"log"
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
		token := ""
		if auth == "" || strings.TrimSpace(auth) == "Bearer" {
			token, _ = c.GetPostForm("access_token")
		} else {
			token = strings.TrimPrefix(auth, "Bearer ")
		}

		if token == "" {
			c.AbortWithStatusJSON(http.StatusUnauthorized, unauthorized)
			log.Println("No required Authorization header")
			return
		}

		if !verifyJwt(token) {
			c.AbortWithStatusJSON(http.StatusUnauthorized, unauthorized)
		}
	}
}
