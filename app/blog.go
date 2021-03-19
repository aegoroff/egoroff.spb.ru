package app

import (
	"fmt"
	"github.com/gin-gonic/gin"
	"net/http"
	"strconv"
)

type Blog struct {
}

func NewBlog() *Blog {
	return &Blog{}
}

func (b *Blog) Route(r *gin.Engine) {
	blog := r.Group("/blog")
	{
		blog.GET("/", func(c *gin.Context) {
			ctx := NewContext("blog", c)
			appContext := ctx["ctx"].(*Context)
			s := appContext.Section("blog")
			ctx["title"] = s.Title
			ctx["poster"] = NewPoster(20)

			c.HTML(http.StatusOK, "blog/index.html", ctx)
		})

		blog.GET("/:page/", func(c *gin.Context) {
			ctx := NewContext("blog", c)
			appContext := ctx["ctx"].(*Context)
			page, _ := strconv.ParseInt(c.Param("page"), 10, 32)

			title := appContext.Section("blog").Title
			if page > 1 {
				title = fmt.Sprintf("%d-я страница", page)
			}
			ctx["title"] = title
			poster := NewPoster(20)

			poster.SetPage(int(page))
			ctx["poster"] = poster

			c.HTML(http.StatusOK, "blog/index.html", ctx)
		})
	}
}
