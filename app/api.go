package app

import (
	"cloud.google.com/go/datastore"
	"github.com/gin-gonic/gin"
	"net/http"
)

type Api struct {
}

func NewApi() *Api {
	return &Api{}
}

func (a *Api) Route(r *gin.Engine) {
	ap := r.Group("/api/v2")
	{
		ap.GET("/posts.json", a.posts)
	}
}

func (a *Api) posts(c *gin.Context) {
	y := c.Query("year")

	q := datastore.NewQuery("Post").Filter("is_public=", true).Order("-created")

	if y != "" {
		m := c.Query("month")
		if m != "" {

		}
	} else {
		tag := c.Query("tag")
		if tag != "" {
			q = q.Filter("tags=", tag)
		}
	}

	adaptor := NewDatastoreAdaptor(q)
	poster := NewCustomPoster(adaptor, 20)

	posts := poster.Posts()

	c.JSON(http.StatusOK, posts)
}
