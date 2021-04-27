package app

import (
	"egoroff.spb.ru/app/framework"
	"fmt"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"strconv"
	"strings"
)

type compatibility struct {
	opinions map[int64]int64
}

func NewCompatibility() Router {
	opinions := map[int64]int64{
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

	return &compatibility{opinions: opinions}
}

func (cm *compatibility) Route(r *gin.Engine) {
	r.GET("/news/", func(c *gin.Context) {
		c.Redirect(http.StatusMovedPermanently, "/blog/")
	})

	r.GET("/apache/", func(c *gin.Context) {
		c.Redirect(http.StatusMovedPermanently, "/portfolio/")
	})

	r.GET("/portfolio/apache/", func(c *gin.Context) {
		c.Redirect(http.StatusMovedPermanently, "/portfolio/")
	})

	r.GET("/portfolio/portfolio/", func(c *gin.Context) {
		c.Redirect(http.StatusMovedPermanently, "/portfolio/")
	})

	r.GET("/portfolio/apache/:doc", func(c *gin.Context) {
		doc := c.Param("doc")
		c.Redirect(http.StatusMovedPermanently, "/portfolio/"+doc)
	})

	r.GET("/portfolio/portfolio/:doc", func(c *gin.Context) {
		doc := c.Param("doc")
		c.Redirect(http.StatusMovedPermanently, "/portfolio/"+doc)
	})

	r.GET("/opinions/", func(c *gin.Context) {
		c.Redirect(http.StatusMovedPermanently, "/blog/")
	})

	r.GET("/opinions/:id", func(c *gin.Context) {
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

		newId, ok := cm.opinions[id]
		if !ok {
			framework.Error404(c)
			return
		}

		c.Redirect(http.StatusMovedPermanently, fmt.Sprintf("/blog/%d.html", newId))
	})
}
