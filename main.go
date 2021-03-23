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

	portfolio := app.NewPortfolio("apache/config.json")
	static := app.NewStaticRouter()
	search := app.NewSearch()
	blog := app.NewBlog()
	api := app.NewApi()
	welcome := app.NewWelcome(portfolio.Documents())

	routers := []app.Router{static, portfolio, welcome, search, blog, api}

	route(r, routers)

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Defaulting to port %s", port)
	err := r.Run(":" + port)
	if err != nil {
		log.Println(err)
	}
}

func route(r *gin.Engine, routers []app.Router) {
	for _, rout := range routers {
		rout.Route(r)
	}
}
