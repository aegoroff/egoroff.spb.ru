package app

import (
	"egoroff.spb.ru/app/auth"
	"egoroff.spb.ru/app/framework"
	"github.com/gin-gonic/gin"
	"net/http"
)

type profile struct {
}

func NewProfile() Router {
	return &profile{}
}

func (s *profile) Route(r *gin.Engine) {
	r.Use(auth.OnlyAuth()).GET("/profile/", func(c *gin.Context) {
		ctx := framework.NewContext(c)
		ctx["title"] = "Редактирование профиля"

		c.HTML(http.StatusOK, "profile.html", ctx)
	})
}
