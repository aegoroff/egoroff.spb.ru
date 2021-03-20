package app

import (
	"cloud.google.com/go/datastore"
	"github.com/gin-gonic/gin"
	"net/http"
	"strconv"
	"time"
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
	page, err := strconv.ParseInt(c.Param("page"), 10, 32)
	if err != nil {
		page = 1
	}
	limit, err := strconv.ParseInt(c.Param("limit"), 10, 32)
	if err != nil {
		limit = 20
	}
	offset, err := strconv.ParseInt(c.Param("limit"), 10, 32)

	q := datastore.NewQuery("Post").Filter("is_public=", true).Order("-created")
	if offset > 0 {
		q = q.Offset(int(offset))
	}

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
	poster := NewCustomPoster(adaptor, int(limit))
	poster.SetPage(int(page))

	posts := poster.Posts()
	for _, post := range posts {
		post.Id = post.Key.ID
	}

	c.JSON(http.StatusOK, ApiResult{
		Status: "success",
		Count:  len(posts),
		Page:   int(page),
		Pages:  poster.PageNums(),
		Now:    time.Now(),
		Result: posts,
	})
}

type ApiResult struct {
	Status string      `json:"status"`
	Count  int         `json:"count"`
	Page   int         `json:"page"`
	Pages  int         `json:"pages"`
	Now    time.Time   `json:"now"`
	Result interface{} `json:"result"`
}
