package app

import (
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"io"
	"log"
	"net/http"
)

// Close wraps io.Closer Close func with error handling
func Close(c io.Closer) {
	if c == nil {
		return
	}
	err := c.Close()
	if err != nil {
		log.Println(err)
	}
}

func error404(c *gin.Context) {
	errorer(c, "404", http.StatusNotFound)
}

func error401(c *gin.Context) {
	errorer(c, "401", http.StatusUnauthorized, Message{
		Type: "danger",
		Text: "You are not authorized to view this content",
	})
}

func errorer(c *gin.Context, title string, code int, messages ...Message) {
	ctx := NewContext(c, messages...)
	ctx["title"] = title
	c.HTML(code, "error.html", ctx)
}

func updateSession(c *gin.Context, action func(s sessions.Session)) {
	session := sessions.Default(c)
	action(session)
	err := session.Save()
	if err != nil {
		log.Println(err)
	}
}
