// Sample storage-quickstart creates a Google Cloud Storage bucket.
package main

import (
	"cloud.google.com/go/datastore"
	"context"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/jbowtie/gokogiri/xml"
	"github.com/jbowtie/ratago/xslt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"
)

func main() {
	r := gin.Default()
	r.LoadHTMLFiles("templates/index.html")
	r.LoadHTMLFiles("templates/apache.html")

	r.GET("/portfolio/apache/:document.html", func(c *gin.Context) {
		doc := c.Param("document.html")
		doc = strings.TrimRight(doc, ".html")

		style, _ := xml.ReadFile("apache/apache_manualpage.xsl", xml.StrictParseOption)
		stylesheet, _ := xslt.ParseStylesheet(style, "apache/apache_manualpage.xsl")

		//process the input
		input, _ := xml.ReadFile(fmt.Sprintf("apache/%s.xml", doc), xml.StrictParseOption)
		output, _ := stylesheet.Process(input, xslt.StylesheetOptions{false, nil})
		c.HTML(http.StatusOK, "apache.html", output)
	})

	r.GET("/", func(c *gin.Context) {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		// Sets your Google Cloud Platform project ID.
		projectID := "egoroff"

		var posts []*Post
		var configs []*Config
		var users []*User
		var files []*File
		var folders []*Folder

		// Create a datastore client. In a typical application, you would create
		// a single client which is reused for every datastore operation.
		dsClient, err := datastore.NewClient(ctx, projectID)
		if err == nil {
			defer dsClient.Close()
			_, err = dsClient.GetAll(ctx, datastore.NewQuery("Post").Order("-created"), &posts)
			if err != nil {
				log.Print(err)
			}
			_, err = dsClient.GetAll(ctx, datastore.NewQuery("Config"), &configs)
			if err != nil {
				log.Print(err)
			}
			_, err = dsClient.GetAll(ctx, datastore.NewQuery("User"), &users)
			if err != nil {
				log.Print(err)
			}
			_, err = dsClient.GetAll(ctx, datastore.NewQuery("File"), &files)
			if err != nil {
				log.Print(err)
			}
			_, err = dsClient.GetAll(ctx, datastore.NewQuery("Folder"), &folders)
			if err != nil {
				log.Print(err)
			}
		} else {
			log.Print(err)
		}

		c.HTML(http.StatusOK, "index.html", gin.H{
			"posts":   posts,
			"configs": configs,
			"users":   users,
		})
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Defaulting to port %s", port)
	r.Run(":" + port)
}
