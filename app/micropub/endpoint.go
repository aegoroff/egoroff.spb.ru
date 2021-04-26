package micropub

import (
	"egoroff.spb.ru/app"
	"egoroff.spb.ru/app/auth/indie"
	"egoroff.spb.ru/app/db"
	"github.com/gin-gonic/gin"
	"net/http"
)

type endpoint struct {
}

func NewEndpoint() app.Router {
	return &endpoint{}
}

func (s *endpoint) Route(r *gin.Engine) {
	repo := db.NewRepository()
	dbAdapter := newStore(repo)
	fileW := db.NewFileWriter()
	geth := getHandler(dbAdapter, indie.ME+"micropub/media", []SyndicateTo{})
	posth := postHandler(dbAdapter, fileW)
	mh := newMediaHandler(fileW)
	mpub := r.Group("/micropub").Use(indie.IndieAuth())
	{
		mpub.GET("/", geth)
		mpub.POST("/", posth)
		mpub.GET("/media", mh.get)
		mpub.POST("/media", mh.post)
		mpub.OPTIONS("/media", func(c *gin.Context) {
			c.Header("Accept", "GET,POST")
			c.Status(http.StatusOK)
		})
	}
}
