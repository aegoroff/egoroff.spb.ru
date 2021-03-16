package main

import (
	"github.com/gin-gonic/gin"
	"github.com/stnc/pongo2gin"
	"log"
	"os"
)

func main() {
	r := gin.Default()
	r.HTMLRender = pongo2gin.TemplatePath("templates")

	styles := []string{
		"min/style/style.min.css",
		"min/style/adminstyle.min.css",
	}

	apacher := newApacher("apache/config.json", styles)
	static := &staticRouter{}
	welcome := newWelcome(apacher.documents, styles)

	routers := []router{static, apacher, welcome}

	route(r, routers)

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Defaulting to port %s", port)
	r.Run(":" + port)
}

func route(r *gin.Engine, routers []router) {
	for _, rout := range routers {
		rout.route(r)
	}
}
