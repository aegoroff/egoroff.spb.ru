package app

import (
	"fmt"
	"github.com/gin-gonic/gin"
	"net/http"
	"sort"
	"strconv"
)

var tagRanks = []string{"tagRank10", "tagRank9",
	"tagRank8", "tagRank7", "tagRank6",
	"tagRank5", "tagRank4", "tagRank3",
	"tagRank2", "tagRank1"}

type Blog struct {
}

type Tag struct {
	Title string
	Level string
}

func NewBlog() *Blog {
	return &Blog{}
}

func (b *Blog) Route(r *gin.Engine) {
	blog := r.Group("/blog")
	{
		blog.GET("/", b.index)
		blog.GET("/:page/", b.index)
	}
}

func (*Blog) index(c *gin.Context) {
	ctx := NewContext("blog", c)
	appContext := ctx["ctx"].(*Context)
	page, err := strconv.ParseInt(c.Param("page"), 10, 32)
	if err != nil {
		page = 1
	}

	rep := NewRepository()
	tags, total := rep.Tags()
	title := appContext.Section("blog").Title
	if page > 1 {
		title = fmt.Sprintf("%d-я страница", page)
	}
	ctx["title"] = title
	poster := NewPoster(20)

	poster.SetPage(int(page))
	ctx["poster"] = poster
	ctx["tags"] = createTags(tags, total)

	c.HTML(http.StatusOK, "blog/index.html", ctx)
}

func createTags(grouped map[string]int, total int) []*Tag {
	var result []*Tag
	for s, i := range grouped {
		index := int(float64(i) / float64(total) * 10)
		result = append(result, &Tag{
			Title: s,
			Level: tagRanks[index],
		})
	}
	sort.Slice(result, func(i, j int) bool {
		return result[i].Title < result[j].Title
	})
	return result
}
