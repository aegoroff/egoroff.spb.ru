package app

import (
	"github.com/gin-gonic/gin"
)

type staticRouter struct{}

func NewStaticRouter() Router {
	return &staticRouter{}
}

func (*staticRouter) Route(r *gin.Engine) {
	r.Static("/p/", "static/")

	// Apache static
	ag := r.Group("/apache")
	{
		ag.Static("images", "apache/images")
		ag.Static("css", "apache/css")
	}

	r.StaticFile("/favicon.ico", "static/img/favicon.ico").
		StaticFile("/robots.txt", "static/robots.txt").
		StaticFile("/googlee53c87e9e3e91020.html", "static/googlee53c87e9e3e91020.html")
}
