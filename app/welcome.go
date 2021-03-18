package app

import (
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
)

type Welcome struct {
	apacheDocs []*Apache
}

func NewWelcome(apacheDocs []*Apache) *Welcome {
	return &Welcome{apacheDocs: apacheDocs}
}

func (w *Welcome) Route(r *gin.Engine) {
	r.GET("/", func(c *gin.Context) {
		repo := NewRepository()
		posts, err := repo.Posts(5)
		if err != nil {
			log.Println(err)
		}

		ctx := NewContext("welcome", c)
		ctx["apache_docs"] = w.apacheDocs
		ctx["posts"] = posts

		c.HTML(http.StatusOK, "welcome.html", ctx)
	})
}
