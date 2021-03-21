package app

import (
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
	ctx := NewContext(c)
	ctx["title"] = "404"
	c.HTML(http.StatusNotFound, "error.html", ctx)
}
