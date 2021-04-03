package main

import (
	"egoroff.spb.ru/app"
	auth2 "egoroff.spb.ru/app/auth"
	"egoroff.spb.ru/app/auth/facebook"
	"egoroff.spb.ru/app/auth/github"
	"egoroff.spb.ru/app/auth/google"
	"egoroff.spb.ru/app/auth/oauth"
	"egoroff.spb.ru/app/blog"
	"github.com/gin-gonic/gin"
	"github.com/stnc/pongo2gin"
	"github.com/gin-contrib/cors"
	"log"
	"os"
)

func main() {
	r := gin.Default()
	r.Use(cors.Default())
	auth2.CreateOrUpdateProviders("static/auth_providers.json")
	oauth.NewStore([]byte("secret"))
	google.Setup()
	github.Setup()
	facebook.Setup()
	r.Use(oauth.Session("goquestsession"))

	r.HTMLRender = pongo2gin.TemplatePath("templates")

	portfolio := app.NewPortfolio("apache/config.json")
	static := app.NewStaticRouter()
	search := app.NewSearch()
	bl := blog.NewBlog()
	api := app.NewApi()
	auth := auth2.NewAuth()
	sitemap := app.NewSiteMap(portfolio.Documents())
	welcome := app.NewWelcome(portfolio.Documents())

	routers := []app.Router{static, portfolio, welcome, search, bl, api, auth, sitemap}

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
