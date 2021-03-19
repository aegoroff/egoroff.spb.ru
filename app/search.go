package app

import (
	"github.com/gin-gonic/gin"
	"net/http"
)

type Search struct {
	Key string
	Cx  string
}

func NewSearch() *Search {
	return &Search{}
}

func (s *Search) Route(r *gin.Engine) {
	r.GET("/search/", func(c *gin.Context) {
		ctx := NewContext("search", c)
		ctx["action_uri"] = "https://www.googleapis.com/customsearch/v1"
		s.Key = ctx["ctx"].(*Context).conf.SearchApiKey
		s.Cx = ctx["ctx"].(*Context).conf.GoogleSiteId
		ctx["search"] = s

		c.HTML(http.StatusOK, "search.html", ctx)
	})
}
