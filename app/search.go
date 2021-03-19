package app

import (
	"github.com/gin-gonic/gin"
	"net/http"
)

type Search struct {
}

func NewSearch() *Search {
	return &Search{}
}

func (s *Search) Route(r *gin.Engine) {
	r.GET("/search/", func(c *gin.Context) {

		ctx := NewContext("search", c)

		c.HTML(http.StatusOK, "search.html", ctx)
	})
}
