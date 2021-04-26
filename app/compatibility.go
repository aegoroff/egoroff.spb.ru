package app

import (
	"github.com/gin-gonic/gin"
	"net/http"
)

type compatibility struct {
	mapping map[string]string
}

func NewCompatibility() Router {
	mapping := map[string]string{
		"/news/":             "/blog/",
		"/opinions/":         "/blog/",
		"/apache/":           "/portfolio/",
		"/portfolio/apache/": "/portfolio/",
	}
	return &compatibility{mapping: mapping}
}

func (c *compatibility) Route(r *gin.Engine) {
	for oldU, newU := range c.mapping {
		r.GET(oldU, func(c *gin.Context) {
			c.Redirect(http.StatusMovedPermanently, newU)
		})
	}
}
