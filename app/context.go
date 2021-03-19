package app

import (
	"encoding/json"
	"github.com/flosch/pongo2"
	"github.com/gin-gonic/gin"
	"log"
	"os"
	"strings"
)

type Context struct {
	conf           *Config
	currentVersion string
	styles         []string
	htmlClass      string
	graph          *Graph
}

func NewContext(htmlClass string, gctx *gin.Context) pongo2.Context {
	styles := []string{
		"min/style/style.min.css",
		//"min/style/adminstyle.min.css",
	}
	repo := NewRepository()
	c, err := repo.Config()
	if err != nil {
		log.Println(err)
	}

	root := readSiteMap()

	gr := NewGraph(root)
	ctx := &Context{
		htmlClass:      htmlClass,
		currentVersion: os.Getenv("CURRENT_VERSION_ID"),
		styles:         styles,
		conf:           c,
		graph:          gr,
	}

	result := pongo2.Context{"ctx": ctx}

	result["sections"] = root.Children

	if gctx.Request.RequestURI != "/" {
		result["breadcrumbs"] = breadcrumbs(gr, gctx.Request.RequestURI)
	}

	return result
}

func (c *Context) PathFor(id string) string {
	return c.graph.FullPath(id)
}

func breadcrumbs(gr *Graph, uri string) []*SiteSection {
	parts := strings.Split(uri, "/")
	root := gr.g.Node(1).(*SiteSection)
	result := []*SiteSection{root}

	for _, part := range parts {
		s := gr.Section(part)
		if s != nil {
			result = append(result, s)
		}
	}
	return result
}

func readSiteMap() *SiteSection {
	fi := NewFiler(os.Stdout)
	b, err := fi.Read("static/map.json")
	if err != nil {
		log.Println(err)
	}

	var root SiteSection
	err = json.Unmarshal(b, &root)
	if err != nil {
		log.Println(err)
	}
	return &root
}
