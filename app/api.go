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
	page, err := strconv.ParseInt(c.Query("page"), 10, 32)
	if err != nil {
		page = 1
	}
	limit, err := strconv.ParseInt(c.Query("limit"), 10, 32)
	if err != nil {
		limit = 20
	}
	offset, err := strconv.ParseInt(c.Query("offset"), 10, 32)

	q := datastore.NewQuery("Post").Filter("is_public=", true).Order("-created")
	if offset > 0 {
		q = q.Offset(int(offset))
	}

	y, _ := strconv.ParseInt(c.Query("year"), 10, 32)
	m, _ := strconv.ParseInt(c.Query("month"), 10, 32)

	if y != 0 {
		loc := time.Now().Location()
		var currMonth time.Time
		var nextMonth time.Time
		if m != 0 {
			currMonth = time.Date(int(y), time.Month(m), 1, 0, 0, 0, 0, loc)
			nextMonth = currMonth.AddDate(0, 1, 0)
		} else {
			currMonth = time.Date(int(y), time.January, 1, 0, 0, 0, 0, loc)
			nextMonth = time.Date(int(y+1), time.January, 1, 0, 0, 0, 0, loc)
		}
		q = q.Filter("created>=", currMonth).Filter("created<", nextMonth)
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
