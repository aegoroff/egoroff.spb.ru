package main

import (
	"cloud.google.com/go/datastore"
	"context"
	"github.com/flosch/pongo2"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"time"
)

type welcome struct {
	apacheDocs []*Apache
}

func newWelcome(apacheDocs []*Apache) *welcome {
	return &welcome{apacheDocs: apacheDocs}
}

func (w *welcome) route(r *gin.Engine) {
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
			"apache_docs": w.apacheDocs,
		})
	})
}
