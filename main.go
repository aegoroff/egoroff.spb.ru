// Sample storage-quickstart creates a Google Cloud Storage bucket.
package main

import (
	"cloud.google.com/go/datastore"
	"context"
	"fmt"
	"github.com/flosch/pongo2"
	"github.com/gin-gonic/gin"
	"github.com/jbowtie/gokogiri/xml"
	"github.com/jbowtie/ratago/xslt"
	"github.com/stnc/pongo2gin"
	"log"
	"net/http"
	"os"
	"strings"
	"time"
)

func main() {
	r := gin.Default()
	r.HTMLRender = pongo2gin.TemplatePath("templates")
	//r.LoadHTMLFiles("templates/index.html")
	//r.LoadHTMLFiles("templates/apache.html")

	r.GET("/portfolio/apache/:document.html", func(c *gin.Context) {
		doc := c.Param("document.html")
		doc = strings.TrimRight(doc, ".html")

		style, _ := xml.ReadFile("apache/apache_manualpage.xsl", xml.StrictParseOption)
		stylesheet, _ := xslt.ParseStylesheet(style, "apache/apache_manualpage.xsl")

		//process the input
		input, _ := xml.ReadFile(fmt.Sprintf("apache/%s.xml", doc), xml.StrictParseOption)
		output, _ := stylesheet.Process(input, xslt.StylesheetOptions{false, nil})
		c.HTML(http.StatusOK, "apache.html", pongo2.Context{
			"content": output,
		})
	})

	r.GET("/", func(c *gin.Context) {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		// Sets your Google Cloud Platform project ID.
		projectID := "egoroff"

		var posts []*Post
		var config Config

		// Create a datastore client. In a typical application, you would create
		// a single client which is reused for every datastore operation.
		dsClient, err := datastore.NewClient(ctx, projectID)
		if err == nil {
			defer dsClient.Close()
			_, err = dsClient.GetAll(ctx, datastore.NewQuery("Post").Order("-created").Limit(5), &posts)
			if err != nil {
				log.Print(err)
			}
			k := datastore.NameKey("Config", "master", nil)
			err = dsClient.Get(ctx, k, &config)
			if err != nil {
				log.Print(err)
			}
		} else {
			log.Print(err)
		}

		c.HTML(http.StatusOK, "welcome.html", pongo2.Context{
			"posts":      posts,
			"html_class": "welcome",
			"config":     config,
		})
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Defaulting to port %s", port)
	r.Run(":" + port)
}
