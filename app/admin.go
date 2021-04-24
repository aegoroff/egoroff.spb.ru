package app

import (
	"egoroff.spb.ru/app/auth"
	"egoroff.spb.ru/app/framework"
	"github.com/gin-gonic/gin"
	"net/http"
)

type admin struct {
}

func NewAdmin() Router {
	return &admin{}
}

func (a *admin) Route(c *gin.Engine) {
	adm := c.Group("/admin").Use(auth.OnlyAdmin())
	{
		adm.GET("/", func(c *gin.Context) {
			ctx := framework.NewContext(c)
			ctx["title"] = "Админка"

			c.HTML(http.StatusOK, "admin.html", ctx)
		})
	}
}
