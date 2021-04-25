package app

import (
	"egoroff.spb.ru/app/auth/indie"
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"
	"fmt"
	"github.com/gin-gonic/gin"
	"net/http"
)

type welcome struct {
	apacheDocs []*domain.Apache
}

func NewWelcome(apacheDocs []*domain.Apache) Router {
	return &welcome{apacheDocs: apacheDocs}
}

func (w *welcome) Route(r *gin.Engine) {
	endpoint := indie.Endpoint
	r.GET("/", func(c *gin.Context) {
		w.addLinkHeader(c, "https://www.egoroff.spb.ru/micropub", "micropub")
		w.addLinkHeader(c, endpoint.AuthURL, "authorization_endpoint")
		w.addLinkHeader(c, endpoint.TokenURL, "token_endpoint")

		poster := db.NewPoster(5)

		ctx := framework.NewContext(c)
		appContext := ctx["ctx"].(*framework.Context)
		ctx["apache_docs"] = w.apacheDocs
		ctx["posts"] = poster.SmallPosts()
		ctx["html_class"] = "welcome"
		ctx["keywords"] = appContext.Section("/").Keywords
		ctx["title"] = appContext.Conf.BrandName

		c.HTML(http.StatusOK, "welcome.html", ctx)
	})
}

func (w *welcome) addLinkHeader(c *gin.Context, link string, rel string) {
	c.Header("Link", fmt.Sprintf("<%s>; rel=\"%s\"", link, rel))
}
