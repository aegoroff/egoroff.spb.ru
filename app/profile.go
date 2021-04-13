package app

import (
	"egoroff.spb.ru/app/framework"
	"github.com/gin-gonic/gin"
	"net/http"
)

type Profile struct {
}

func NewProfile() Router {
	return &Profile{}
}

func (s *Profile) Route(r *gin.Engine) {
	r.GET("/profile/", func(c *gin.Context) {
		ctx := framework.NewContext(c)
		ctx["title"] = "Редактирование профиля"

		c.HTML(http.StatusOK, "profile.html", ctx)
	})
}
