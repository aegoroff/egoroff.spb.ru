package blog

import (
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/gomarkdown/markdown"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"
)

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

func NewBlog() *Blog {
	return &Blog{}
}

func (b *Blog) Route(r *gin.Engine) {
	r.GET("/news/rss", b.atom)
	r.GET("/recent.atom", b.atom)
	blog := r.Group("/blog")
	{
		blog.GET("/", b.index)
		blog.GET("/recent.atom", b.atom)
		blog.GET("/:id", b.post)
		blog.GET("/page/:page", b.index)
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

	c.HTML(http.StatusOK, "blog/index.html", ctx)
}

func (*Blog) atom(c *gin.Context) {
	poster := db.NewPoster(20)
	poster.SetPage(1)

	links := []domain.FeedLink{
		{
			Href: "http://www.egoroff.spb.ru/",
		},
		{
			Href: "http://www.egoroff.spb.ru/blog/recent.atom",
			Rel:  "self",
		},
	}

	posts := poster.Posts()
	var entries []domain.FeedEntry

	for _, post := range posts {
		e := domain.FeedEntry{
			XmlBase: "http://www.egoroff.spb.ru/blog/recent.atom",
			Title: domain.FeedTitle{
				Title: post.Title,
				Type:  "text",
			},
			Id:        fmt.Sprintf("http://www.egoroff.spb.ru/blog/%v.html", post.Key.ID),
			Updated:   post.Created.Format(time.RFC3339),
			Published: post.Created.Format(time.RFC3339),
			Author: domain.FeedAuthor{
				Name: "Alexander Egorov",
			},
		}
		content := domain.FeedContent{
			Content: post.Html(),
			Type:    "html",
		}
		e.Content = content
		entries = append(entries, e)
	}

	feed := domain.Feed{
		XmlNS: "http://www.w3.org/2005/Atom",
		Title: domain.FeedTitle{
			Title: "egoroff.spb.ru feed",
			Type:  "text",
		},
		Id:      "http://www.egoroff.spb.ru/blog/recent.atom",
		Updated: posts[0].Created.Format(time.RFC3339),
		Link:    links,
		Entries: entries,
	}

	c.XML(http.StatusOK, feed)
}

func (b *Blog) post(c *gin.Context) {
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

	if post.Markdown {
		md := []byte(post.Text)
		ctx["content"] = string(markdown.ToHTML(md, nil, nil))
	} else if post.Text != "" && strings.HasPrefix(post.Text, "<?xml version=\"1.0\"?>") {
		ctx["content"] = convert(post.Text)
	} else {
		ctx["content"] = post.Text
	}

	ctx["main_post"] = post
	ctx["html_class"] = "blog"
	ctx["keywords"] = strings.Join(post.Tags, ",")

	c.HTML(http.StatusOK, "blog/post.html", ctx)
}
