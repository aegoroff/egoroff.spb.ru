// Sample storage-quickstart creates a Google Cloud Storage bucket.
package main

import (
	"cloud.google.com/go/datastore"
	"context"
	"encoding/json"
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

	apache_docs := create_apache_docs()

	apache_docs_map := make(map[string]*Apache)
	for _, doc := range apache_docs {
		apache_docs_map[doc.ID] = doc
	}

	r.Static("/apache/images", "apache/images").
		Static("/apache/css", "apache/css").
		Static("/p/", "static/")

	r.StaticFile("/favicon.ico", "static/img/favicon.ico").
		StaticFile("/robots.txt", "static/robots.txt").
		StaticFile("/googlee53c87e9e3e91020.html", "static/googlee53c87e9e3e91020.html")

	r.GET("/portfolio/apache/:document.html", func(c *gin.Context) {
		doc := c.Param("document.html")
		doc = strings.TrimRight(doc, ".html")

		apache, ok := apache_docs_map[doc]

		if !ok {
			return
		}

		sheet := fmt.Sprintf("apache/%s.xsl", apache.Stylesheet)
		style, _ := xml.ReadFile(sheet, xml.StrictParseOption)
		stylesheet, _ := xslt.ParseStylesheet(style, sheet)

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
			"posts":       posts,
			"html_class":  "welcome",
			"config":      config,
			"apache_docs": apache_docs,
		})
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Defaulting to port %s", port)
	r.Run(":" + port)
}

func create_apache_docs() []*Apache {
	f, err := os.Open("apache/config.json")
	if err != nil {
		log.Println(err)
	}
	dec := json.NewDecoder(f)
	for {
		var v map[string][]string
		if err := dec.Decode(&v); err != nil {
			log.Println(err)
			return nil
		}
		var result = make([]*Apache, 0, len(v))
		for k, val := range v {
			a := Apache{
				ID:          k,
				Stylesheet:  val[0],
				Title:       val[1],
				Description: val[2],
				Keywords:    val[3],
			}
			result = append(result, &a)
		}
		return result
	}
}
