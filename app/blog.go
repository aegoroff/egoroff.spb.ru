package app

import (
	"fmt"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"sort"
	"strconv"
	"strings"
	"time"
)

var tagRanks = []string{"tagRank10", "tagRank9",
	"tagRank8", "tagRank7", "tagRank6",
	"tagRank5", "tagRank4", "tagRank3",
	"tagRank2", "tagRank1"}

var months = []string{"Январь", "Февраль",
	"Март", "Апрель", "Май",
	"Июнь", "Июль", "Август",
	"Сентябрь", "Октябрь", "Ноябрь", "Декабрь"}

type Blog struct {
}

type Tag struct {
	Title string
	Level string
}

type Year struct {
	Year   int
	Posts  int
	Months []*Month
}

type Month struct {
	Month int
	Name  string
	Posts int
}

func NewBlog() *Blog {
	return &Blog{}
}

func (b *Blog) Route(r *gin.Engine) {
	blog := r.Group("/blog")
	{
		blog.GET("/", b.index)
		blog.GET("/:id", b.post)
		blog.GET("/:id/:page", b.index)
	}
}

func (*Blog) index(c *gin.Context) {
	ctx := NewContext("blog", c)
	appContext := ctx["ctx"].(*Context)
	id := c.Param("id")

	page, err := strconv.ParseInt(id, 10, 32)
	if err != nil {
		p := c.Param("page")
		page, err = strconv.ParseInt(p, 10, 32)
		if err != nil {
			page = 1
		}
	}

	rep := NewRepository()
	containers := rep.Tags()
	tags, times, total := groupContainers(containers)
	title := appContext.Section("blog").Title
	if page > 1 {
		title = fmt.Sprintf("%d-я страница", page)
	}
	ctx["title"] = title
	poster := NewPoster(20)

	poster.SetPage(int(page))
	ctx["poster"] = poster
	ctx["tags"] = createTags(tags, total)
	ctx["archive"] = createArchive(times)

	c.HTML(http.StatusOK, "blog/index.html", ctx)
}

func (b *Blog) post(c *gin.Context) {
	ids := c.Param("id")
	if !strings.HasSuffix(ids, ".html") {
		b.index(c)
		return
	}
	id, err := strconv.ParseInt(ids[:len(ids)-len(".html")], 10, 64)
	if err != nil {
		log.Println(err)
		ctx := NewContext("", c)
		ctx["title"] = "404"
		c.HTML(http.StatusNotFound, "error.html", ctx)
		return
	}

	rep := NewRepository()
	post := rep.Post(id)

	if post == nil || post.Key == nil {
		log.Println(err)
		ctx := NewContext("", c)
		ctx["title"] = "404"
		c.HTML(http.StatusNotFound, "error.html", ctx)
		return
	}

	ctx := NewContext("blog", c)
	ctx["title"] = post.Title
	ctx["content"] = post.Text
	ctx["main_post"] = post

	c.HTML(http.StatusOK, "blog/post.html", ctx)
}

func groupContainers(containers []*TagContainter) (map[string]int, map[int64]time.Time, int) {
	tags := make(map[string]int)
	dates := make(map[int64]time.Time)
	for _, tc := range containers {
		dates[tc.Key.ID] = tc.Created
		for _, t := range tc.Tags {
			tags[t] += 1
		}
	}

	return tags, dates, len(containers)
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

func createArchive(times map[int64]time.Time) []*Year {
	var result []*Year
	byYear := make(map[int]*Year)
	for _, date := range times {
		y := date.Year()
		m := int(date.Month())
		item, ok := byYear[y]
		if !ok {
			item = &Year{Year: y, Months: []*Month{
				{
					Month: m,
					Name:  months[m-1],
				},
			}}
			byYear[y] = item
		}
		item.Posts += 1
		monthAdded := false
		for _, month := range item.Months {
			if month.Month == m {
				month.Posts += 1
				monthAdded = true
				break
			}
		}
		if !monthAdded {
			item.Months = append(item.Months, &Month{
				Month: m,
				Name:  months[m-1],
				Posts: 1,
			})
		}
	}
	for _, year := range byYear {
		result = append(result, year)
		sort.Slice(year.Months, func(i, j int) bool {
			return year.Months[i].Month > year.Months[j].Month
		})
	}

	sort.Slice(result, func(i, j int) bool {
		return result[i].Year > result[j].Year
	})
	return result
}
