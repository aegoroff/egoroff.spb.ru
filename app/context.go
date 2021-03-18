package app

import (
	"encoding/json"
	"github.com/flosch/pongo2"
	"github.com/gin-gonic/gin"
	"log"
	"os"
)

type Context struct {
	conf           *Config
	currentVersion string
	styles         []string
	htmlClass      string
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
	sections := readSiteMap()

	var root *SiteSection
	for _, section := range sections {
		if section.Id == "/" {
			root = section
			break
		}
	}

	ctx := &Context{
		htmlClass:      htmlClass,
		currentVersion: os.Getenv("CURRENT_VERSION_ID"),
		styles:         styles,
		conf:           c,
	}

	result := pongo2.Context{
		"ctx": ctx,
	}

	if gctx.Request.RequestURI != "/" {
		result["breadcrumbs"] = []*SiteSection{root}
	}

	return result
}

func readSiteMap() []*SiteSection {
	fi := NewFiler(os.Stdout)
	b, err := fi.Read("static/map.json")
	if err != nil {
		log.Println(err)
	}

	var sections []*SiteSection
	err = json.Unmarshal(b, &sections)
	if err != nil {
		log.Println(err)
	}
	return sections
}
