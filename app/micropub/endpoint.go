package micropub

import (
	"egoroff.spb.ru/app"
	"egoroff.spb.ru/app/auth/indie"
	"egoroff.spb.ru/app/db"
	"github.com/gin-gonic/gin"
)

type endpoint struct {
}

func NewEndpoint() app.Router {
	return &endpoint{}
}

func (s *endpoint) Route(r *gin.Engine) {
	repo := db.NewRepository()
	dbAdapter := newStore(repo)
	geth := getHandler(dbAdapter, "/media", []SyndicateTo{})
	posth := postHandler(dbAdapter)
	mpub := r.Group("/micropub").Use(indie.IndieAuth())
	{
		mpub.GET("/", geth)
		mpub.POST("/", posth)
	}
}
