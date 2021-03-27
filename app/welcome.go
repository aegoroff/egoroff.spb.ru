package app

import (
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"
	"github.com/gin-gonic/gin"
	"github.com/vcraescu/go-paginator/v2"
	"log"
	"net/http"
)

type Welcome struct {
	apacheDocs []*domain.Apache
}

func NewWelcome(apacheDocs []*domain.Apache) *Welcome {
	return &Welcome{apacheDocs: apacheDocs}
}

func (w *Welcome) Route(r *gin.Engine) {
	r.GET("/", func(c *gin.Context) {
		pager := paginator.New(db.NewPostsAdaptor(), 5)
		var posts []*domain.Post
		err := pager.Results(&posts)
		if err != nil {
			log.Println(err)
		}

		ctx := framework.NewContext(c)
		appContext := ctx["ctx"].(*framework.Context)
		ctx["apache_docs"] = w.apacheDocs
		ctx["posts"] = posts
		ctx["html_class"] = "welcome"
		ctx["keywords"] = appContext.Section("/").Keywords
		ctx["title"] = appContext.Conf.BrandName

		c.HTML(http.StatusOK, "welcome.html", ctx)
	})
}
