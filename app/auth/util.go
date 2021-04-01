package auth

import (
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"log"
)

func UpdateSession(c *gin.Context, action func(s sessions.Session)) {
	session := sessions.Default(c)
	action(session)
	err := session.Save()
	if err != nil {
		log.Println(err)
	}
}
