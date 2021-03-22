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
		ctx := NewContext(c)
		ctx["action_uri"] = "https://www.googleapis.com/customsearch/v1"
		context := ctx["ctx"].(*Context)
		conf := context.conf
		s.Key = conf.SearchApiKey
		s.Cx = conf.GoogleSiteId
		ctx["search"] = s
		section := context.Section("search")
		ctx["title"] = section.Title
		ctx["meta_description"] = section.Descr
		ctx["html_class"] = "search"

		c.HTML(http.StatusOK, "search.html", ctx)
	})
}
