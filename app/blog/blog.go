package blog

import (
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"
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

var Remapping = map[int64]int64{
	1:  19002,
	3:  24001,
	5:  23001,
	6:  21002,
	7:  22001,
	8:  21001,
	9:  13004,
	10: 6007,
	11: 18001,
	12: 12004,
	13: 12003,
	14: 1006,
	15: 6005,
	16: 16002,
	17: 12002,
	18: 6006,
	19: 17001,
	21: 11004,
	22: 9004,
	23: 13003,
	24: 16001,
	25: 9003,
	26: 14005,
	27: 5004,
	28: 11003,
	29: 15001,
	30: 8002,
}

var opinionsRemapping = map[int64]int64{
	1:  25002,
	4:  31001,
	8:  6003,
	11: 30001,
	13: 3006,
	18: 29001,
	21: 9002,
	22: 2004,
	24: 25003,
	25: 22002,
	26: 27002,
	27: 27001,
	28: 14004,
	29: 8003,
	30: 6004,
}

type Blog struct {
}

type Tag struct {
	Title string `json:"title"`
	Level string `json:"level"`
}

type Year struct {
	Year   int      `json:"year"`
	Posts  int      `json:"posts"`
	Months []*Month `json:"months"`
}

type Month struct {
	Month int    `json:"month"`
	Name  string `json:"name"`
	Posts int    `json:"posts"`
}

type Archive struct {
	Tags  interface{}  `json:"tags"`
	Years interface{} `json:"years"`
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
	r.GET("/opinions/:id", b.opinion)
}

func (*Blog) index(c *gin.Context) {
	ctx := framework.NewContext(c)
	ctx["html_class"] = "blog"
	appContext := ctx["ctx"].(*framework.Context)
	id := c.Param("id")

	page, err := strconv.ParseInt(id, 10, 32)
	if err != nil {
		p := c.Param("page")
		page, err = strconv.ParseInt(p, 10, 32)
		if err != nil {
			page = 1
		}
	}

	rep := db.NewRepository()
	containers := rep.Tags()
	tags, times, total := groupContainers(containers)
	section := appContext.Section("blog")
	title := section.Title
	if page > 1 {
		title = fmt.Sprintf("%d-я страница", page)
	}
	ctx["title"] = title
	ctx["keywords"] = section.Keywords
	ctx["meta_description"] = section.Descr
	poster := db.NewPoster(20)

	poster.SetPage(int(page))
	ctx["poster"] = poster
	ctx["tags"] = createTags(tags, total)
	ctx["archive"] = createArchive(times)

	c.HTML(http.StatusOK, "blog/index.html", ctx)
}

func (b *Blog) post(c *gin.Context) {
	ids := c.Param("id")
	if strings.EqualFold(ids, "archive") {
		b.archive(c)
		return
	}
	if !strings.HasSuffix(ids, ".html") {
		b.index(c)
		return
	}
	id, err := strconv.ParseInt(ids[:len(ids)-len(".html")], 10, 64)
	if err != nil {
		log.Println(err)
		framework.Error404(c)
		return
	}

	if remapped, ok := Remapping[id]; ok {
		id = remapped
	}

	b.showPost(c, id)
}

func (b *Blog) opinion(c *gin.Context) {
	ids := c.Param("id")
	if !strings.HasSuffix(ids, ".html") {
		framework.Error404(c)
		return
	}

	id, err := strconv.ParseInt(ids[:len(ids)-len(".html")], 10, 64)
	if err != nil {
		log.Println(err)
		framework.Error404(c)
		return
	}

	if remapped, ok := opinionsRemapping[id]; ok {
		id = remapped
	} else {
		framework.Error404(c)
		return
	}

	b.showPost(c, id)
}

func (b *Blog) showPost(c *gin.Context, id int64) {
	rep := db.NewRepository()
	post := rep.Post(id)

	if post == nil || post.Key == nil {
		framework.Error404(c)
		return
	}

	ctx := framework.NewContext(c)
	ctx["title"] = post.Title

	if post.Text != "" && strings.HasPrefix(post.Text, "<?xml version=\"1.0\"?>") {
		ctx["content"] = convert(post.Text)
	} else {
		ctx["content"] = post.Text
	}

	ctx["main_post"] = post
	ctx["html_class"] = "blog"
	ctx["keywords"] = strings.Join(post.Tags, ",")

	c.HTML(http.StatusOK, "blog/post.html", ctx)
}

func (*Blog) archive(c *gin.Context) {
	rep := db.NewRepository()
	containers := rep.Tags()
	tags, times, total := groupContainers(containers)
	t := createTags(tags, total)
	y := createArchive(times)
	a := Archive{
		Tags:  t,
		Years: y,
	}
	c.JSON(http.StatusOK, a)
}

func groupContainers(containers []*domain.TagContainter) (map[string]int, map[int64]time.Time, int) {
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
