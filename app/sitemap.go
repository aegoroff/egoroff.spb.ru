package app

import (
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"
	"github.com/gin-gonic/gin"
	"net/http"
	"strconv"
)

const site = "https://www.egoroff.spb.ru/"

type siteMap struct {
	apacheDocs []*domain.Apache
}

func NewSiteMap(apacheDocs []*domain.Apache) Router {
	return &siteMap{apacheDocs: apacheDocs}
}

func (s *siteMap) Route(r *gin.Engine) {
	r.GET("/sitemap.xml", func(c *gin.Context) {
		sm := framework.ReadSiteMap()
		var urls []domain.Url
		urls = append(urls, domain.Url{
			Location:   site,
			ChangeFreq: "weekly",
			Priority:   1.0,
		})

		for _, child := range sm.Children {
			urls = append(urls, domain.Url{
				Location:   site + child.Id + "/",
				ChangeFreq: "weekly",
				Priority:   0.7,
			})
		}

		for _, doc := range s.apacheDocs {
			urls = append(urls, domain.Url{
				Location:   site + "portfolio/" + doc.ID + ".html",
				ChangeFreq: "yearly",
				Priority:   1.0,
			})
		}
		rep := db.NewRepository()
		posts := rep.PostKeys()
		for _, post := range posts {
			urls = append(urls, domain.Url{
				Location:   site + "blog/" + strconv.FormatInt(post.ID, 10) + ".html",
				ChangeFreq: "yearly",
				Priority:   1.0,
			})
		}

		urlset := domain.UrlSet{
			XmlNS: "http://www.sitemaps.org/schemas/sitemap/0.9",
			Urls:  urls,
		}

		c.XML(http.StatusOK, urlset)
	})
}
