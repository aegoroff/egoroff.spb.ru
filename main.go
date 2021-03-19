package main

import (
	"egoroff.spb.ru/app"
	"github.com/gin-gonic/gin"
	"github.com/stnc/pongo2gin"
	"log"
	"os"
)

func main() {
	r := gin.Default()
	r.HTMLRender = pongo2gin.TemplatePath("templates")

	apacher := app.NewApacher("apache/config.json")
	static := app.NewStaticRouter()
	search := app.NewSearch()
	welcome := app.NewWelcome(apacher.Documents())

	routers := []app.Router{static, apacher, welcome, search}

	route(r, routers)

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Defaulting to port %s", port)
	r.Run(":" + port)
}

func route(r *gin.Engine, routers []app.Router) {
	for _, rout := range routers {
		rout.Route(r)
	}
}
