package auth

import (
	"crypto/rand"
	"encoding/base64"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"log"
)

func randToken() string {
	b := make([]byte, 32)
	_, err := rand.Read(b)
	if err != nil {
		log.Println(err)
	}
	return base64.StdEncoding.EncodeToString(b)
}

func updateSession(c *gin.Context, action func(s sessions.Session)) {
	session := sessions.Default(c)
	action(session)
	err := session.Save()
	if err != nil {
		log.Println(err)
	}
}
