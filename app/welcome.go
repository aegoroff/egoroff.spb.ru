package app

import (
	"github.com/gin-gonic/gin"
	"github.com/vcraescu/go-paginator/v2"
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
		pager := paginator.New(NewPostsAdaptor(), 5)
		var posts []*Post
		err := pager.Results(&posts)
		if err != nil {
			log.Println(err)
		}

		ctx := NewContext(c)
		appContext := ctx["ctx"].(*Context)
		ctx["apache_docs"] = w.apacheDocs
		ctx["posts"] = posts
		ctx["html_class"] = "welcome"
		ctx["keywords"] = appContext.Section("/").Keywords
		ctx["title"] = appContext.conf.BrandName

		c.HTML(http.StatusOK, "welcome.html", ctx)
	})
}
