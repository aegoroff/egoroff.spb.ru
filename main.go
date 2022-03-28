package main

import (
	"egoroff.spb.ru/app"
	auth2 "egoroff.spb.ru/app/auth"
	"egoroff.spb.ru/app/auth/github"
	"egoroff.spb.ru/app/auth/google"
	"egoroff.spb.ru/app/auth/indie"
	"egoroff.spb.ru/app/auth/oauth"
	"egoroff.spb.ru/app/blog"
	"egoroff.spb.ru/app/micropub"
	"egoroff.spb.ru/app/txt"
	"github.com/flosch/pongo2"
	"github.com/gin-contrib/cors"
	"github.com/gin-contrib/gzip"
	"github.com/gin-gonic/gin"
	"github.com/stnc/pongo2gin"
	"log"
	"os"
)

func main() {
	r := gin.Default()
	r.Use(cors.Default())
	r.Use(gzip.Gzip(gzip.DefaultCompression))
	auth2.CreateOrUpdateProviders("static/auth_providers.json")
	oauth.NewStore([]byte("secret"))
	google.Setup()
	github.Setup()
	r.Use(oauth.Session("goquestsession"))

	r.HTMLRender = pongo2gin.TemplatePath("static/dist")
	err := pongo2.RegisterFilter("typograph", typograph)
	if err != nil {
		log.Println(err)
	}

	portfolio := app.NewPortfolio("apache/config.json")
	static := app.NewStaticRouter()
	search := app.NewSearch()
	profile := app.NewProfile()
	down := app.NewDownload()
	indieAuth := indie.NewAuth()
	indieToken := indie.NewTokenEndpoint(indieAuth)
	bl := blog.NewBlog()
	mpub := micropub.NewEndpoint()
	api := app.NewApi()
	auth := auth2.NewAuth()
	admin := app.NewAdmin()
	sitemap := app.NewSiteMap(portfolio.Documents())
	welcome := app.NewWelcome(portfolio.Documents())
	compat := app.NewCompatibility()

	routers := []app.Router{
		static,
		portfolio,
		welcome,
		search,
		bl,
		api,
		auth,
		sitemap,
		profile,
		down,
		mpub,
		indieAuth,
		indieToken,
		admin,
		compat,
	}

	route(r, routers)

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Defaulting to port %s", port)
	err = r.Run(":" + port)
	if err != nil {
		log.Println(err)
	}
}

func route(r *gin.Engine, routers []app.Router) {
	for _, rout := range routers {
		rout.Route(r)
	}
}

func typograph(in *pongo2.Value, _ *pongo2.Value) (*pongo2.Value, *pongo2.Error) {
	return pongo2.AsValue(txt.ParseHtml(in.String())), nil
}
