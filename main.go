package main

import (
	"egoroff.spb.ru/app"
	"github.com/gin-gonic/gin"
	"github.com/stnc/pongo2gin"
	"github.com/zalando/gin-oauth2/google"
	"log"
	"os"
)

func main() {
	scopes := []string{
		"https://www.googleapis.com/auth/userinfo.email",
		// You have to select your own scope from here -> https://developers.google.com/identity/protocols/googlescopes#google_sign-in
	}
	secret := []byte("secret")
	sessionName := "goquestsession"

	r := gin.Default()
	google.Setup("https://4-dot-egoroff.appspot.com/_s/callback/google/authorized/", "static/site_credentials.json", scopes, secret)
	r.Use(google.Session(sessionName))

	r.HTMLRender = pongo2gin.TemplatePath("templates")

	portfolio := app.NewPortfolio("apache/config.json")
	static := app.NewStaticRouter()
	search := app.NewSearch()
	blog := app.NewBlog()
	api := app.NewApi()
	auth := app.NewAuth()
	welcome := app.NewWelcome(portfolio.Documents())

	routers := []app.Router{static, portfolio, welcome, search, blog, api, auth}

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
